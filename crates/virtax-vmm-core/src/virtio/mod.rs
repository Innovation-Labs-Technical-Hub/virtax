//! The virtio 1.2 modern PCI transport (virtio 1.2 §4.1): vendor capabilities pointing into one
//! memory BAR that holds the common, ISR, device and notification registers and the MSI-X table.
//!
//! Queue notifications are ioeventfds on the notify registers, so a guest kick wakes the device's
//! worker thread without a vCPU exit. A device implements [`VirtioDevice`]; the transport handles
//! feature negotiation, device status, queue setup and reset.

pub mod block;

use std::sync::{Arc, Mutex};

use virtio_queue::{Queue, QueueT};
use vmm_sys_util::eventfd::EventFd;

use crate::pci::msix::{Msix, NO_VECTOR};
use crate::pci::{ConfigSpace, Ids, PciFunction, Platform};

/// `VIRTIO_F_VERSION_1`: the modern interface, required of every device here.
pub const F_VERSION_1: u64 = 1 << 32;

/// Device status bits (virtio 1.2 §2.1).
const STATUS_FEATURES_OK: u8 = 8;
const STATUS_DRIVER_OK: u8 = 4;
const STATUS_FAILED: u8 = 128;

/// PCI vendor capability types (virtio 1.2 §4.1.4).
const CAP_COMMON: u8 = 1;
const CAP_NOTIFY: u8 = 2;
const CAP_ISR: u8 = 3;
const CAP_DEVICE: u8 = 4;
const VENDOR_CAPABILITY: u8 = 0x09;

/// The layout of BAR 0.
const BAR: usize = 0;
const BAR_SIZE: u32 = 0x8000;
const COMMON_OFFSET: u64 = 0x0000;
const COMMON_SIZE: u64 = 0x40;
const ISR_OFFSET: u64 = 0x1000;
const DEVICE_OFFSET: u64 = 0x2000;
const DEVICE_SIZE: u64 = 0x1000;
const NOTIFY_OFFSET: u64 = 0x3000;
const NOTIFY_MULTIPLIER: u32 = 4;
const MSIX_TABLE_OFFSET: u64 = 0x4000;
const MSIX_PBA_OFFSET: u64 = 0x5000;

const VIRTIO_VENDOR: u16 = 0x1af4;

/// A virtio device behind the PCI transport.
pub trait VirtioDevice: Send {
    /// The virtio device ID (2 for a block device).
    fn device_id(&self) -> u16;
    /// PCI class code for the function, `0xCCSSPP`.
    fn pci_class(&self) -> u32;
    /// Features the device offers, including [`F_VERSION_1`].
    fn features(&self) -> u64;
    /// Read the device-specific configuration.
    fn read_config(&self, offset: u64, data: &mut [u8]);
    /// One maximum size per queue.
    fn queue_max_sizes(&self) -> &[u16];
    /// The event a guest kick of queue `index` signals.
    fn queue_event(&self, index: usize) -> &EventFd;
    /// The driver set DRIVER_OK: process `queues` (each with its MSI-X vector) from now on.
    fn activate(&mut self, queues: Vec<(Queue, u16)>, msix: Arc<Mutex<Msix>>);
    /// The driver reset the device: stop using the queues.
    fn reset(&mut self);
}

#[derive(Default, Clone)]
struct QueueRegisters {
    size: u16,
    vector: u16,
    enabled: bool,
    desc: u64,
    driver: u64,
    device: u64,
}

/// One virtio device as a PCI function.
pub struct VirtioPci {
    config: ConfigSpace,
    msix_capability: usize,
    msix: Arc<Mutex<Msix>>,
    device: Box<dyn VirtioDevice>,
    platform: Arc<dyn Platform>,
    device_feature_select: u32,
    driver_feature_select: u32,
    driver_features: u64,
    config_vector: u16,
    status: u8,
    queue_select: u16,
    queues: Vec<QueueRegisters>,
    /// Where BAR 0 decodes, while its notify registers are ioeventfds.
    mapped: Option<u64>,
}

/// `struct virtio_pci_cap` after the ID and next bytes, plus the notify multiplier when given.
fn virtio_capability(kind: u8, offset: u64, length: u64, multiplier: Option<u32>) -> Vec<u8> {
    let cap_len = if multiplier.is_some() { 20 } else { 16 };
    let mut body = vec![cap_len, kind, BAR as u8, 0, 0, 0];
    body.extend((offset as u32).to_le_bytes());
    body.extend((length as u32).to_le_bytes());
    if let Some(m) = multiplier {
        body.extend(m.to_le_bytes());
    }
    body
}

impl VirtioPci {
    pub fn new(
        device: Box<dyn VirtioDevice>,
        platform: Arc<dyn Platform>,
    ) -> std::io::Result<Self> {
        let queues = device.queue_max_sizes().len();
        let mut config = ConfigSpace::new(Ids {
            vendor: VIRTIO_VENDOR,
            device: 0x1040 + device.device_id(),
            revision: 1,
            class: device.pci_class(),
            subsystem_vendor: VIRTIO_VENDOR,
            subsystem: 0x40,
        });
        config.add_bar(BAR, BAR_SIZE);
        for (kind, offset, length, multiplier) in [
            (CAP_COMMON, COMMON_OFFSET, COMMON_SIZE, None),
            (CAP_ISR, ISR_OFFSET, 4, None),
            (CAP_DEVICE, DEVICE_OFFSET, DEVICE_SIZE, None),
            (
                CAP_NOTIFY,
                NOTIFY_OFFSET,
                u64::from(NOTIFY_MULTIPLIER) * queues as u64,
                Some(NOTIFY_MULTIPLIER),
            ),
        ] {
            config.add_capability(
                VENDOR_CAPABILITY,
                &virtio_capability(kind, offset, length, multiplier),
                &[],
            );
        }
        // One vector for configuration changes and one per queue.
        let msix = Msix::new(queues as u16 + 1, Arc::clone(&platform))?;
        let msix_capability = msix.add_capability(
            &mut config,
            BAR as u8,
            MSIX_TABLE_OFFSET as u32,
            MSIX_PBA_OFFSET as u32,
        );
        let mut transport = Self {
            config,
            msix_capability,
            msix: Arc::new(Mutex::new(msix)),
            device,
            platform,
            device_feature_select: 0,
            driver_feature_select: 0,
            driver_features: 0,
            config_vector: NO_VECTOR,
            status: 0,
            queue_select: 0,
            queues: Vec::new(),
            mapped: None,
        };
        transport.reset();
        Ok(transport)
    }

    fn reset(&mut self) {
        self.device.reset();
        self.device_feature_select = 0;
        self.driver_feature_select = 0;
        self.driver_features = 0;
        self.config_vector = NO_VECTOR;
        self.status = 0;
        self.queue_select = 0;
        self.queues = self
            .device
            .queue_max_sizes()
            .iter()
            .map(|&max| QueueRegisters {
                size: max,
                vector: NO_VECTOR,
                ..Default::default()
            })
            .collect();
    }

    fn selected(&mut self) -> Option<&mut QueueRegisters> {
        self.queues.get_mut(usize::from(self.queue_select))
    }

    fn common_read(&mut self, offset: u64) -> u64 {
        let features = self.device.features();
        let q = self.queues.get(usize::from(self.queue_select)).cloned();
        match offset {
            0x00 => u64::from(self.device_feature_select),
            0x04 => match self.device_feature_select {
                0 => features & 0xffff_ffff,
                1 => features >> 32,
                _ => 0,
            },
            0x08 => u64::from(self.driver_feature_select),
            0x0c => match self.driver_feature_select {
                0 => self.driver_features & 0xffff_ffff,
                1 => self.driver_features >> 32,
                _ => 0,
            },
            0x10 => u64::from(self.config_vector),
            0x12 => self.queues.len() as u64,
            0x14 => u64::from(self.status),
            0x15 => 0, // configuration generation: the device configuration never changes
            0x16 => u64::from(self.queue_select),
            0x18 => q.map_or(0, |q| u64::from(q.size)),
            0x1a => q.map_or(u64::from(NO_VECTOR), |q| u64::from(q.vector)),
            0x1c => q.map_or(0, |q| u64::from(q.enabled)),
            0x1e => u64::from(self.queue_select), // queue_notify_off: one register per queue
            0x20..=0x37 => q.map_or(0, |q| {
                let field = [q.desc, q.driver, q.device][((offset - 0x20) / 8) as usize];
                field >> (8 * ((offset - 0x20) % 8))
            }),
            0x38 => u64::from(self.queue_select), // queue_notif_config_data
            _ => 0,                               // queue_reset (not offered) and padding
        }
    }

    fn common_write(&mut self, offset: u64, value: u64, len: usize) {
        // 64-bit fields are written as two 32-bit halves.
        let set_half = |field: &mut u64, at: u64| {
            if len == 8 {
                *field = value;
            } else if at == 0 {
                *field = (*field & !0xffff_ffff) | (value & 0xffff_ffff);
            } else {
                *field = (*field & 0xffff_ffff) | (value << 32);
            }
        };
        match offset {
            0x00 => self.device_feature_select = value as u32,
            0x08 => self.driver_feature_select = value as u32,
            0x0c => match self.driver_feature_select {
                0 => self.driver_features = (self.driver_features & !0xffff_ffff) | value,
                1 => self.driver_features = (self.driver_features & 0xffff_ffff) | (value << 32),
                _ => {}
            },
            0x10 => self.config_vector = value as u16,
            0x14 => self.write_status(value as u8),
            0x16 => self.queue_select = value as u16,
            0x18 => {
                let max = self
                    .device
                    .queue_max_sizes()
                    .get(usize::from(self.queue_select))
                    .copied()
                    .unwrap_or(0);
                if let Some(q) = self.selected() {
                    // Split queues need a power-of-two size no larger than the maximum.
                    let size = value as u16;
                    if size.is_power_of_two() && size <= max {
                        q.size = size;
                    }
                }
            }
            0x1a => {
                if let Some(q) = self.selected() {
                    q.vector = value as u16;
                }
            }
            0x1c => {
                if let Some(q) = self.selected() {
                    q.enabled = value == 1;
                }
            }
            0x20..=0x37 => {
                let field = (offset - 0x20) / 8;
                let at = (offset - 0x20) % 8;
                if let Some(q) = self.selected() {
                    let target = match field {
                        0 => &mut q.desc,
                        1 => &mut q.driver,
                        _ => &mut q.device,
                    };
                    set_half(target, at);
                }
            }
            _ => {}
        }
    }

    fn write_status(&mut self, status: u8) {
        if status == 0 {
            self.reset();
            return;
        }
        let mut status = status;
        if status & STATUS_FEATURES_OK != 0 && self.status & STATUS_FEATURES_OK == 0 {
            let offered = self.device.features();
            let acceptable =
                self.driver_features & !offered == 0 && self.driver_features & F_VERSION_1 != 0;
            if !acceptable {
                // The driver reads FEATURES_OK back and gives up.
                status &= !STATUS_FEATURES_OK;
            }
        }
        if status & STATUS_DRIVER_OK != 0 && self.status & STATUS_DRIVER_OK == 0 {
            match self.build_queues() {
                Ok(queues) => self.device.activate(queues, Arc::clone(&self.msix)),
                Err(e) => {
                    tracing::warn!(error = %e, "virtio driver set up an invalid queue");
                    status |= STATUS_FAILED;
                }
            }
        }
        self.status = status;
    }

    fn build_queues(&self) -> Result<Vec<(Queue, u16)>, String> {
        let maxes = self.device.queue_max_sizes();
        self.queues
            .iter()
            .zip(maxes)
            .enumerate()
            .filter(|(_, (q, _))| q.enabled)
            .map(|(i, (q, &max))| {
                let mut queue = Queue::new(max).map_err(|e| format!("queue {i}: {e:?}"))?;
                queue.set_size(q.size);
                queue.set_desc_table_address(Some(q.desc as u32), Some((q.desc >> 32) as u32));
                queue.set_avail_ring_address(Some(q.driver as u32), Some((q.driver >> 32) as u32));
                queue.set_used_ring_address(Some(q.device as u32), Some((q.device >> 32) as u32));
                queue.set_ready(true);
                Ok((queue, q.vector))
            })
            .collect()
    }

    fn notify_address(base: u64, queue: usize) -> u64 {
        base + NOTIFY_OFFSET + u64::from(NOTIFY_MULTIPLIER) * queue as u64
    }
}

fn read_le(data: &[u8]) -> u64 {
    let mut b = [0u8; 8];
    b[..data.len().min(8)].copy_from_slice(&data[..data.len().min(8)]);
    u64::from_le_bytes(b)
}

impl PciFunction for VirtioPci {
    fn config(&self) -> &ConfigSpace {
        &self.config
    }

    fn config_mut(&mut self) -> &mut ConfigSpace {
        &mut self.config
    }

    fn config_written(&mut self, offset: usize, len: usize) {
        let control = self.msix_capability + 2;
        if offset < control + 2 && offset + len > control {
            self.msix
                .lock()
                .unwrap()
                .update_control(&self.config, self.msix_capability);
        }
    }

    fn bar_read(&mut self, _bar: usize, offset: u64, data: &mut [u8]) {
        match offset {
            o if o < COMMON_OFFSET + COMMON_SIZE => {
                let value = self.common_read(o - COMMON_OFFSET);
                let bytes = value.to_le_bytes();
                data.copy_from_slice(&bytes[..data.len()]);
            }
            // Interrupts are MSI-X only, so the ISR status is never set.
            o if (ISR_OFFSET..ISR_OFFSET + 4).contains(&o) => data.fill(0),
            o if (DEVICE_OFFSET..DEVICE_OFFSET + DEVICE_SIZE).contains(&o) => {
                self.device.read_config(o - DEVICE_OFFSET, data)
            }
            o if (MSIX_TABLE_OFFSET..MSIX_PBA_OFFSET).contains(&o) => self
                .msix
                .lock()
                .unwrap()
                .table_read(o - MSIX_TABLE_OFFSET, data),
            o if o >= MSIX_PBA_OFFSET => self
                .msix
                .lock()
                .unwrap()
                .pba_read(o - MSIX_PBA_OFFSET, data),
            _ => data.fill(0),
        }
    }

    fn bar_write(&mut self, _bar: usize, offset: u64, data: &[u8]) {
        match offset {
            o if o < COMMON_OFFSET + COMMON_SIZE => {
                self.common_write(o - COMMON_OFFSET, read_le(data), data.len())
            }
            o if (NOTIFY_OFFSET..MSIX_TABLE_OFFSET).contains(&o) => {
                // Reaches here only while no ioeventfd covers the register.
                let queue = ((o - NOTIFY_OFFSET) / u64::from(NOTIFY_MULTIPLIER)) as usize;
                if queue < self.queues.len() {
                    let _ = self.device.queue_event(queue).write(1);
                }
            }
            o if (MSIX_TABLE_OFFSET..MSIX_PBA_OFFSET).contains(&o) => self
                .msix
                .lock()
                .unwrap()
                .table_write(o - MSIX_TABLE_OFFSET, data),
            _ => {}
        }
    }

    fn bar_mapped(&mut self, _bar: usize, addr: Option<u64>) {
        let queues = self.queues.len();
        if let Some(old) = self.mapped.take() {
            for q in 0..queues {
                let event = self.device.queue_event(q);
                if let Err(e) = self
                    .platform
                    .unregister_mmio_ioevent(event, Self::notify_address(old, q))
                {
                    tracing::warn!(error = %e, "cannot remove a virtio notification");
                }
            }
        }
        if let Some(new) = addr {
            for q in 0..queues {
                let event = self.device.queue_event(q);
                if let Err(e) = self
                    .platform
                    .register_mmio_ioevent(event, Self::notify_address(new, q))
                {
                    tracing::warn!(error = %e, "cannot register a virtio notification; kicks will exit to the VMM");
                }
            }
            self.mapped = Some(new);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pci::tests::FakePlatform;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A device with one queue of 8 that records activation.
    struct Probe {
        event: EventFd,
        activated: Arc<AtomicUsize>,
    }

    impl VirtioDevice for Probe {
        fn device_id(&self) -> u16 {
            2
        }
        fn pci_class(&self) -> u32 {
            0x01_80_00
        }
        fn features(&self) -> u64 {
            F_VERSION_1 | 1 << 9
        }
        fn read_config(&self, offset: u64, data: &mut [u8]) {
            data.fill(offset as u8 + 1);
        }
        fn queue_max_sizes(&self) -> &[u16] {
            &[8]
        }
        fn queue_event(&self, _: usize) -> &EventFd {
            &self.event
        }
        fn activate(&mut self, queues: Vec<(Queue, u16)>, _: Arc<Mutex<Msix>>) {
            assert_eq!(queues.len(), 1);
            let (q, vector) = &queues[0];
            assert_eq!((q.size(), q.desc_table(), *vector), (4, 0x1_0000_2000, 1));
            self.activated.fetch_add(1, Ordering::SeqCst);
        }
        fn reset(&mut self) {}
    }

    fn transport() -> (VirtioPci, Arc<AtomicUsize>, Arc<FakePlatform>) {
        let activated = Arc::new(AtomicUsize::new(0));
        let platform = Arc::new(FakePlatform::default());
        let probe = Probe {
            event: EventFd::new(0).unwrap(),
            activated: Arc::clone(&activated),
        };
        (
            VirtioPci::new(Box::new(probe), platform.clone()).unwrap(),
            activated,
            platform,
        )
    }

    fn w(t: &mut VirtioPci, offset: u64, value: u32, len: usize) {
        t.bar_write(BAR, offset, &value.to_le_bytes()[..len]);
    }

    fn r(t: &mut VirtioPci, offset: u64, len: usize) -> u32 {
        let mut d = [0u8; 4];
        t.bar_read(BAR, offset, &mut d[..len]);
        u32::from_le_bytes(d)
    }

    #[test]
    fn capabilities_point_into_bar_0() {
        let (t, _, _) = transport();
        let mut kinds = Vec::new();
        let mut at = t.config.u16(0x34) as usize & 0xff;
        while at != 0 {
            let mut cap = [0u8; 16];
            t.config.read(at, &mut cap);
            if cap[0] == VENDOR_CAPABILITY {
                kinds.push((
                    cap[3],
                    cap[4],
                    u32::from_le_bytes(cap[8..12].try_into().unwrap()),
                ));
            }
            at = cap[1] as usize;
        }
        assert_eq!(
            kinds,
            vec![
                (CAP_COMMON, 0, 0x0000),
                (CAP_ISR, 0, 0x1000),
                (CAP_DEVICE, 0, 0x2000),
                (CAP_NOTIFY, 0, 0x3000)
            ]
        );
        assert_eq!(t.config.u16(2), 0x1042, "modern virtio-blk device ID");
    }

    #[test]
    fn negotiates_features_and_activates_on_driver_ok() {
        let (mut t, activated, _) = transport();
        w(&mut t, 0x00, 1, 4);
        assert_eq!(r(&mut t, 0x04, 4), 1, "VERSION_1 in the high word");
        w(&mut t, 0x00, 0, 4);
        assert_eq!(r(&mut t, 0x04, 4), 1 << 9);
        assert_eq!(r(&mut t, 0x12, 2), 1, "one queue");

        // A driver that does not accept VERSION_1 is refused.
        w(&mut t, 0x14, 1 | 2 | 8, 1);
        assert_eq!(r(&mut t, 0x14, 1) & 8, 0, "FEATURES_OK withheld");
        w(&mut t, 0x14, 0, 1);

        w(&mut t, 0x08, 1, 4);
        w(&mut t, 0x0c, 1, 4);
        w(&mut t, 0x08, 0, 4);
        w(&mut t, 0x0c, 1 << 9, 4);
        w(&mut t, 0x14, 1 | 2 | 8, 1);
        assert_eq!(r(&mut t, 0x14, 1), 1 | 2 | 8);

        w(&mut t, 0x16, 0, 2);
        assert_eq!(r(&mut t, 0x18, 2), 8, "maximum size by default");
        w(&mut t, 0x18, 6, 2);
        assert_eq!(r(&mut t, 0x18, 2), 8, "not a power of two");
        w(&mut t, 0x18, 4, 2);
        w(&mut t, 0x1a, 1, 2);
        w(&mut t, 0x20, 0x2000, 4);
        w(&mut t, 0x24, 1, 4);
        w(&mut t, 0x28, 0x3000, 4);
        w(&mut t, 0x30, 0x4000, 4);
        assert_eq!(r(&mut t, 0x24, 4), 1, "high half of the descriptor address");
        w(&mut t, 0x1c, 1, 2);
        w(&mut t, 0x14, 1 | 2 | 8 | 4, 1);
        assert_eq!(activated.load(Ordering::SeqCst), 1);

        // Reset clears the negotiated state.
        w(&mut t, 0x14, 0, 1);
        assert_eq!(r(&mut t, 0x14, 1), 0);
        assert_eq!(r(&mut t, 0x1c, 2), 0);
    }

    #[test]
    fn device_configuration_and_kicks() {
        let (mut t, _, platform) = transport();
        assert_eq!(r(&mut t, DEVICE_OFFSET + 4, 1), 5);
        // Without an ioeventfd, a kick written through the MMIO exit still signals the queue.
        w(&mut t, NOTIFY_OFFSET, 0, 2);
        assert_eq!(t.device.queue_event(0).read().unwrap(), 1);
        t.bar_mapped(BAR, Some(0xc000_0000));
        assert_eq!(*platform.ioevents.lock().unwrap(), vec![0xc000_3000]);
        t.bar_mapped(BAR, Some(0xc001_0000));
        assert_eq!(*platform.ioevents.lock().unwrap(), vec![0xc001_3000]);
        t.bar_mapped(BAR, None);
        assert!(platform.ioevents.lock().unwrap().is_empty());
    }
}
