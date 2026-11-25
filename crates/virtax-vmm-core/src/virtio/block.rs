//! virtio-blk (virtio 1.2 §5.2) on a raw image: one request queue, served by a worker thread that
//! reads and writes the image straight into guest memory and makes flushes durable with
//! `fdatasync` (writeback caching).

use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::sync::{Arc, Mutex};
use std::thread;

use virtio_queue::{DescriptorChain, Queue, QueueT};
use vm_memory::{Address, Bytes, GuestAddress, GuestMemoryMmap};
use vmm_sys_util::eventfd::EventFd;

use super::{F_VERSION_1, VirtioDevice};
use crate::disk::{DiskImage, SECTOR_SIZE};
use crate::pci::msix::Msix;

pub const DEVICE_ID: u16 = 2;
const QUEUE_SIZE: u16 = 256;

// Feature bits (virtio 1.2 §5.2.3).
const F_SEG_MAX: u64 = 1 << 2;
const F_RO: u64 = 1 << 5;
const F_BLK_SIZE: u64 = 1 << 6;
const F_FLUSH: u64 = 1 << 9;

// Request types and status values (virtio 1.2 §5.2.6).
const T_IN: u32 = 0;
const T_OUT: u32 = 1;
const T_FLUSH: u32 = 4;
const T_GET_ID: u32 = 8;
const S_OK: u8 = 0;
const S_IOERR: u8 = 1;
const S_UNSUPP: u8 = 2;
const HEADER_SIZE: usize = 16;
const ID_BYTES: usize = 20;

/// The queue while the driver has the device running.
struct Active {
    queue: Queue,
    vector: u16,
    msix: Arc<Mutex<Msix>>,
}

/// A virtio-blk device. Its worker thread lives as long as the process.
pub struct Block {
    event: EventFd,
    features: u64,
    config: Vec<u8>,
    active: Arc<Mutex<Option<Active>>>,
}

/// What the worker owns: the image and the guest memory it transfers into.
struct Worker {
    file: File,
    size: u64,
    read_only: bool,
    id: [u8; ID_BYTES],
    memory: Arc<GuestMemoryMmap>,
}

impl Block {
    /// A block device for `disk`; `id` is the serial the guest reads with GET_ID (truncated to 20
    /// bytes).
    pub fn new(disk: DiskImage, id: &str, memory: Arc<GuestMemoryMmap>) -> std::io::Result<Self> {
        let mut features = F_VERSION_1 | F_SEG_MAX | F_BLK_SIZE | F_FLUSH;
        if disk.read_only {
            features |= F_RO;
        }
        let mut config = vec![0u8; 24];
        config[..8].copy_from_slice(&(disk.size / SECTOR_SIZE).to_le_bytes()); // capacity
        // size_max (8) stays 0: no limit. seg_max: every descriptor but the header and status.
        config[12..16].copy_from_slice(&u32::from(QUEUE_SIZE - 2).to_le_bytes());
        config[20..24].copy_from_slice(&(SECTOR_SIZE as u32).to_le_bytes()); // blk_size
        let mut id_bytes = [0u8; ID_BYTES];
        let n = id.len().min(ID_BYTES);
        id_bytes[..n].copy_from_slice(&id.as_bytes()[..n]);

        let event = EventFd::new(0)?;
        let kick = event.try_clone()?;
        let active: Arc<Mutex<Option<Active>>> = Arc::new(Mutex::new(None));
        let mut worker = Worker {
            file: disk.file,
            size: disk.size,
            read_only: disk.read_only,
            id: id_bytes,
            memory,
        };
        let shared = Arc::clone(&active);
        thread::Builder::new()
            .name("virtio-blk".into())
            .spawn(move || {
                while kick.read().is_ok() {
                    if let Some(active) = shared.lock().unwrap().as_mut() {
                        worker.process(active);
                    }
                }
            })?;
        Ok(Self {
            event,
            features,
            config,
            active,
        })
    }
}

impl VirtioDevice for Block {
    fn device_id(&self) -> u16 {
        DEVICE_ID
    }

    fn pci_class(&self) -> u32 {
        0x01_80_00 // mass storage, other
    }

    fn features(&self) -> u64 {
        self.features
    }

    fn read_config(&self, offset: u64, data: &mut [u8]) {
        for (i, b) in data.iter_mut().enumerate() {
            *b = self.config.get(offset as usize + i).copied().unwrap_or(0);
        }
    }

    fn queue_max_sizes(&self) -> &[u16] {
        &[QUEUE_SIZE]
    }

    fn queue_event(&self, _index: usize) -> &EventFd {
        &self.event
    }

    fn activate(&mut self, mut queues: Vec<(Queue, u16)>, msix: Arc<Mutex<Msix>>) {
        let Some((queue, vector)) = queues.pop() else {
            return;
        };
        *self.active.lock().unwrap() = Some(Active {
            queue,
            vector,
            msix,
        });
        // Serve anything the driver queued before DRIVER_OK.
        let _ = self.event.write(1);
    }

    fn reset(&mut self) {
        // Waits for a request batch in progress to finish.
        *self.active.lock().unwrap() = None;
    }
}

/// Guest buffers of one request, in order.
type Buffers = Vec<(GuestAddress, usize)>;

/// Remove the first `n` bytes of `buffers` and return them as buffers of their own.
fn take_front(buffers: &mut Buffers, mut n: usize) -> Option<Buffers> {
    let mut out = Vec::new();
    while n > 0 {
        let (addr, len) = *buffers.first()?;
        if len <= n {
            out.push((addr, len));
            buffers.remove(0);
            n -= len;
        } else {
            out.push((addr, n));
            buffers[0] = (addr.unchecked_add(n as u64), len - n);
            n = 0;
        }
    }
    Some(out)
}

/// Remove the last byte of `buffers` and return its address.
fn take_last_byte(buffers: &mut Buffers) -> Option<GuestAddress> {
    let (addr, len) = buffers.pop()?;
    if len > 1 {
        buffers.push((addr, len - 1));
    }
    Some(addr.unchecked_add(len as u64 - 1))
}

fn total(buffers: &Buffers) -> usize {
    buffers.iter().map(|(_, len)| len).sum()
}

impl Worker {
    fn process(&mut self, active: &mut Active) {
        let memory = Arc::clone(&self.memory);
        let mem = memory.as_ref();
        if !active.queue.is_valid(mem) {
            tracing::warn!("virtio-blk queue addresses are outside guest memory");
            return;
        }
        let mut completed = false;
        while let Some(chain) = active.queue.pop_descriptor_chain(mem) {
            let head = chain.head_index();
            let used = self.request(mem, chain);
            if let Err(e) = active.queue.add_used(mem, head, used) {
                tracing::warn!(error = ?e, "virtio-blk cannot complete a request");
            }
            completed = true;
        }
        if completed && active.queue.needs_notification(mem).unwrap_or(true) {
            active.msix.lock().unwrap().notify(active.vector);
        }
    }

    /// Serve one request; returns the number of bytes written into guest memory.
    fn request(&mut self, mem: &GuestMemoryMmap, chain: DescriptorChain<&GuestMemoryMmap>) -> u32 {
        let (mut readable, mut writable): (Buffers, Buffers) = (Vec::new(), Vec::new());
        for desc in chain {
            let buffer = (desc.addr(), desc.len() as usize);
            if desc.is_write_only() {
                writable.push(buffer);
            } else {
                readable.push(buffer);
            }
        }
        let Some(status_at) = take_last_byte(&mut writable) else {
            tracing::warn!("virtio-blk request without a status byte");
            return 0;
        };
        let (status, written) = match self.serve(mem, &mut readable, &writable) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(error = %e, "virtio-blk request failed");
                (S_IOERR, 0)
            }
        };
        if mem.write_obj(status, status_at).is_err() {
            return 0;
        }
        written as u32 + 1
    }

    /// Returns the status and how many data bytes went to the guest.
    fn serve(
        &mut self,
        mem: &GuestMemoryMmap,
        readable: &mut Buffers,
        writable: &Buffers,
    ) -> Result<(u8, usize), String> {
        let mut header = [0u8; HEADER_SIZE];
        let mut filled = 0;
        for (addr, len) in take_front(readable, HEADER_SIZE).ok_or("request header too short")? {
            mem.read_slice(&mut header[filled..filled + len], addr)
                .map_err(|e| e.to_string())?;
            filled += len;
        }
        let kind = u32::from_le_bytes(header[..4].try_into().unwrap());
        let sector = u64::from_le_bytes(header[8..16].try_into().unwrap());
        match kind {
            T_IN => {
                let offset = self.check_range(sector, total(writable))?;
                self.file
                    .seek(SeekFrom::Start(offset))
                    .map_err(|e| e.to_string())?;
                for &(addr, len) in writable {
                    mem.read_exact_volatile_from(addr, &mut self.file, len)
                        .map_err(|e| e.to_string())?;
                }
                Ok((S_OK, total(writable)))
            }
            T_OUT if self.read_only => Ok((S_IOERR, 0)),
            T_OUT => {
                let offset = self.check_range(sector, total(readable))?;
                self.file
                    .seek(SeekFrom::Start(offset))
                    .map_err(|e| e.to_string())?;
                for &(addr, len) in readable.iter() {
                    mem.write_all_volatile_to(addr, &mut self.file, len)
                        .map_err(|e| e.to_string())?;
                }
                Ok((S_OK, 0))
            }
            T_FLUSH if self.read_only => Ok((S_OK, 0)),
            T_FLUSH => {
                self.file.sync_data().map_err(|e| e.to_string())?;
                Ok((S_OK, 0))
            }
            T_GET_ID => {
                let mut left = &self.id[..];
                let mut written = 0;
                for &(addr, len) in writable {
                    let n = len.min(left.len());
                    mem.write_slice(&left[..n], addr)
                        .map_err(|e| e.to_string())?;
                    left = &left[n..];
                    written += n;
                }
                Ok((S_OK, written))
            }
            _ => Ok((S_UNSUPP, 0)),
        }
    }

    fn check_range(&self, sector: u64, len: usize) -> Result<u64, String> {
        let offset = sector
            .checked_mul(SECTOR_SIZE)
            .ok_or("sector number overflows")?;
        match offset.checked_add(len as u64) {
            Some(end) if end <= self.size => Ok(offset),
            _ => Err(format!(
                "request for {len} bytes at sector {sector} is past the end of the disk"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::disk::DiskConfig;
    use crate::pci::tests::FakePlatform;
    use std::time::{Duration, Instant};

    const DESC: u64 = 0x1000;
    const AVAIL: u64 = 0x2000;
    const USED: u64 = 0x3000;
    const SIZE: u16 = 8;
    const WRITE_ONLY: u16 = 2;
    const NEXT: u16 = 1;

    struct Rig {
        mem: Arc<GuestMemoryMmap>,
        block: Block,
        submitted: u16,
        _dir: tempfile::TempDir,
        path: std::path::PathBuf,
    }

    fn rig(read_only: bool) -> Rig {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("disk.img");
        std::fs::write(&path, vec![0u8; 64 * 1024]).unwrap();
        let disk = DiskImage::open(&DiskConfig {
            path: path.clone(),
            read_only,
        })
        .unwrap();
        let mem = Arc::new(GuestMemoryMmap::from_ranges(&[(GuestAddress(0), 1 << 20)]).unwrap());
        let mut block = Block::new(disk, "test-serial", Arc::clone(&mem)).unwrap();
        let mut queue = Queue::new(SIZE).unwrap();
        queue.set_desc_table_address(Some(DESC as u32), Some(0));
        queue.set_avail_ring_address(Some(AVAIL as u32), Some(0));
        queue.set_used_ring_address(Some(USED as u32), Some(0));
        queue.set_ready(true);
        let msix = Msix::new(2, Arc::new(FakePlatform::default())).unwrap();
        block.activate(vec![(queue, 1)], Arc::new(Mutex::new(msix)));
        Rig {
            mem,
            block,
            submitted: 0,
            _dir: dir,
            path,
        }
    }

    impl Rig {
        fn desc(&self, index: u16, addr: u64, len: u32, flags: u16) {
            let at = GuestAddress(DESC + 16 * u64::from(index));
            self.mem.write_obj(addr, at).unwrap();
            self.mem.write_obj(len, at.unchecked_add(8)).unwrap();
            self.mem.write_obj(flags, at.unchecked_add(12)).unwrap();
            self.mem.write_obj(index + 1, at.unchecked_add(14)).unwrap();
        }

        /// Submit header + `data` buffers + status as one chain; returns the status address.
        fn submit(&mut self, kind: u32, sector: u64, data: &[(u64, u32, bool)]) -> u64 {
            let header = 0x10_000;
            self.mem.write_obj(kind, GuestAddress(header)).unwrap();
            self.mem
                .write_obj(sector, GuestAddress(header + 8))
                .unwrap();
            self.desc(0, header, HEADER_SIZE as u32, NEXT);
            for (i, &(addr, len, guest_writes)) in data.iter().enumerate() {
                let flags = NEXT | if guest_writes { WRITE_ONLY } else { 0 };
                self.desc(i as u16 + 1, addr, len, flags);
            }
            let status = 0x20_000;
            self.mem.write_obj(0xffu8, GuestAddress(status)).unwrap();
            self.desc(data.len() as u16 + 1, status, 1, WRITE_ONLY);
            let slot = u64::from(self.submitted % SIZE);
            self.mem
                .write_obj(0u16, GuestAddress(AVAIL + 4 + 2 * slot))
                .unwrap();
            self.submitted += 1;
            self.mem
                .write_obj(self.submitted, GuestAddress(AVAIL + 2))
                .unwrap();
            self.block.queue_event(0).write(1).unwrap();
            let deadline = Instant::now() + Duration::from_secs(5);
            while self.mem.read_obj::<u16>(GuestAddress(USED + 2)).unwrap() != self.submitted {
                assert!(Instant::now() < deadline, "request not completed");
                thread::sleep(Duration::from_millis(1));
            }
            status
        }

        fn status(&self, at: u64) -> u8 {
            self.mem.read_obj(GuestAddress(at)).unwrap()
        }

        fn used_len(&self) -> u32 {
            let slot = u64::from((self.submitted - 1) % SIZE);
            self.mem
                .read_obj(GuestAddress(USED + 4 + 8 * slot + 4))
                .unwrap()
        }
    }

    #[test]
    fn advertises_capacity_and_features() {
        let r = rig(false);
        let mut cap = [0u8; 8];
        r.block.read_config(0, &mut cap);
        assert_eq!(u64::from_le_bytes(cap), 128, "64 KiB in sectors");
        assert_eq!(r.block.features() & F_RO, 0);
        assert_ne!(r.block.features() & F_FLUSH, 0);
        assert_ne!(rig(true).block.features() & F_RO, 0);
    }

    #[test]
    fn writes_flushes_and_reads_back_through_the_image() {
        let mut r = rig(false);
        // A write split over two buffers, at sector 2.
        r.mem
            .write_slice(&[0xab; 512], GuestAddress(0x40_000))
            .unwrap();
        r.mem
            .write_slice(&[0xcd; 512], GuestAddress(0x50_000))
            .unwrap();
        let s = r.submit(T_OUT, 2, &[(0x40_000, 512, false), (0x50_000, 512, false)]);
        assert_eq!(r.status(s), S_OK);
        let s = r.submit(T_FLUSH, 0, &[]);
        assert_eq!(r.status(s), S_OK);
        let file = std::fs::read(&r.path).unwrap();
        assert!(file[1024..1536].iter().all(|b| *b == 0xab));
        assert!(file[1536..2048].iter().all(|b| *b == 0xcd));
        assert!(file[..1024].iter().all(|b| *b == 0), "nothing else changed");

        let s = r.submit(T_IN, 3, &[(0x60_000, 512, true)]);
        assert_eq!(r.status(s), S_OK);
        assert_eq!(r.used_len(), 513, "data and status");
        let mut back = [0u8; 512];
        r.mem.read_slice(&mut back, GuestAddress(0x60_000)).unwrap();
        assert_eq!(back, [0xcd; 512]);
    }

    #[test]
    fn get_id_out_of_range_and_unknown_requests() {
        let mut r = rig(false);
        let s = r.submit(T_GET_ID, 0, &[(0x60_000, 20, true)]);
        assert_eq!(r.status(s), S_OK);
        let mut id = [0u8; 11];
        r.mem.read_slice(&mut id, GuestAddress(0x60_000)).unwrap();
        assert_eq!(&id, b"test-serial");
        let s = r.submit(T_IN, 127, &[(0x60_000, 1024, true)]);
        assert_eq!(r.status(s), S_IOERR, "past the end");
        let s = r.submit(99, 0, &[]);
        assert_eq!(r.status(s), S_UNSUPP);
    }

    #[test]
    fn a_read_only_disk_refuses_writes() {
        let mut r = rig(true);
        r.mem
            .write_slice(&[0xee; 512], GuestAddress(0x40_000))
            .unwrap();
        let s = r.submit(T_OUT, 0, &[(0x40_000, 512, false)]);
        assert_eq!(r.status(s), S_IOERR);
        assert!(std::fs::read(&r.path).unwrap().iter().all(|b| *b == 0));
        let s = r.submit(T_IN, 0, &[(0x60_000, 512, true)]);
        assert_eq!(r.status(s), S_OK, "reads still work");
    }

    #[test]
    fn buffer_splitting() {
        let mut b: Buffers = vec![(GuestAddress(0), 10), (GuestAddress(100), 10)];
        assert_eq!(
            take_front(&mut b, 12).unwrap(),
            vec![(GuestAddress(0), 10), (GuestAddress(100), 2)]
        );
        assert_eq!(b, vec![(GuestAddress(102), 8)]);
        assert_eq!(take_last_byte(&mut b), Some(GuestAddress(109)));
        assert_eq!(b, vec![(GuestAddress(102), 7)]);
        assert!(take_front(&mut b, 8).is_none());
    }
}
