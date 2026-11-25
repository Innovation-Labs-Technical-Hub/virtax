//! MSI-X (PCI Local Bus 3.0 §6.8.2): the capability, the vector table and the pending-bit array.
//!
//! Each vector owns a GSI with an irqfd. Delivering a vector routes its GSI to the message the guest
//! programmed (only when that message changed) and writes the irqfd; a masked vector sets its pending
//! bit instead and is delivered when the guest unmasks it.

use std::sync::Arc;

use vmm_sys_util::eventfd::EventFd;

use super::{ConfigSpace, Platform};

pub const CAPABILITY_ID: u8 = 0x11;
const CONTROL_ENABLE: u16 = 1 << 15;
const CONTROL_FUNCTION_MASK: u16 = 1 << 14;
const ENTRY_SIZE: u64 = 16;
const VECTOR_MASKED: u32 = 1;
/// "No vector" in virtio's MSI-X vector registers.
pub const NO_VECTOR: u16 = 0xffff;

struct Vector {
    address: u64,
    data: u32,
    control: u32,
    gsi: u32,
    irq: EventFd,
    routed: Option<(u64, u32)>,
}

impl Vector {
    fn bytes(&self) -> [u8; ENTRY_SIZE as usize] {
        let mut b = [0u8; ENTRY_SIZE as usize];
        b[..8].copy_from_slice(&self.address.to_le_bytes());
        b[8..12].copy_from_slice(&self.data.to_le_bytes());
        b[12..].copy_from_slice(&self.control.to_le_bytes());
        b
    }

    fn set_bytes(&mut self, b: &[u8; ENTRY_SIZE as usize]) {
        self.address = u64::from_le_bytes(b[..8].try_into().unwrap());
        self.data = u32::from_le_bytes(b[8..12].try_into().unwrap());
        self.control = u32::from_le_bytes(b[12..].try_into().unwrap());
    }

    fn masked(&self) -> bool {
        self.control & VECTOR_MASKED != 0
    }
}

/// The MSI-X state of one function, shared between its MMIO handlers and its I/O worker.
pub struct Msix {
    vectors: Vec<Vector>,
    pending: u64,
    enabled: bool,
    function_masked: bool,
    platform: Arc<dyn Platform>,
}

impl Msix {
    /// `count` vectors (at most 64), each with its own GSI and irqfd.
    pub fn new(count: u16, platform: Arc<dyn Platform>) -> std::io::Result<Self> {
        assert!((1..=64).contains(&count), "1 to 64 MSI-X vectors");
        let mut vectors = Vec::with_capacity(count.into());
        for _ in 0..count {
            let gsi = platform.allocate_gsi();
            let irq = EventFd::new(0)?;
            platform.register_irqfd(&irq, gsi)?;
            vectors.push(Vector {
                address: 0,
                data: 0,
                // Vectors start masked.
                control: VECTOR_MASKED,
                gsi,
                irq,
                routed: None,
            });
        }
        Ok(Self {
            vectors,
            pending: 0,
            enabled: false,
            function_masked: false,
            platform,
        })
    }

    pub fn count(&self) -> u16 {
        self.vectors.len() as u16
    }

    /// Add the capability to `config`, with the table at `table_offset` and the PBA at `pba_offset`
    /// of BAR `bar`. Returns the capability's offset.
    pub fn add_capability(
        &self,
        config: &mut ConfigSpace,
        bar: u8,
        table_offset: u32,
        pba_offset: u32,
    ) -> usize {
        let mut body = Vec::with_capacity(10);
        body.extend((self.count() - 1).to_le_bytes());
        body.extend((table_offset | u32::from(bar)).to_le_bytes());
        body.extend((pba_offset | u32::from(bar)).to_le_bytes());
        let mut writable = vec![0u8; 10];
        writable[..2].copy_from_slice(&(CONTROL_ENABLE | CONTROL_FUNCTION_MASK).to_le_bytes());
        config.add_capability(CAPABILITY_ID, &body, &writable)
    }

    /// Follow the message control register after a configuration write.
    pub fn update_control(&mut self, config: &ConfigSpace, capability: usize) {
        let control = config.u16(capability + 2);
        self.enabled = control & CONTROL_ENABLE != 0;
        self.function_masked = control & CONTROL_FUNCTION_MASK != 0;
        self.deliver_pending();
    }

    pub fn table_read(&self, offset: u64, data: &mut [u8]) {
        for (i, b) in data.iter_mut().enumerate() {
            let at = offset + i as u64;
            *b = self
                .vectors
                .get((at / ENTRY_SIZE) as usize)
                .map_or(0, |v| v.bytes()[(at % ENTRY_SIZE) as usize]);
        }
    }

    pub fn table_write(&mut self, offset: u64, data: &[u8]) {
        let index = (offset / ENTRY_SIZE) as usize;
        let start = (offset % ENTRY_SIZE) as usize;
        let Some(vector) = self.vectors.get_mut(index) else {
            return;
        };
        if start + data.len() > ENTRY_SIZE as usize {
            return; // accesses never cross an entry
        }
        let mut bytes = vector.bytes();
        bytes[start..start + data.len()].copy_from_slice(data);
        vector.set_bytes(&bytes);
        self.deliver_pending();
    }

    pub fn pba_read(&self, offset: u64, data: &mut [u8]) {
        let bytes = self.pending.to_le_bytes();
        for (i, b) in data.iter_mut().enumerate() {
            *b = bytes.get(offset as usize + i).copied().unwrap_or(0);
        }
    }

    /// Signal `vector`; does nothing for [`NO_VECTOR`] or while MSI-X is disabled.
    pub fn notify(&mut self, vector: u16) {
        let index = usize::from(vector);
        if vector == NO_VECTOR || index >= self.vectors.len() || !self.enabled {
            return;
        }
        if self.function_masked || self.vectors[index].masked() {
            self.pending |= 1 << index;
        } else {
            self.fire(index);
        }
    }

    fn deliver_pending(&mut self) {
        if !self.enabled || self.function_masked {
            return;
        }
        for index in 0..self.vectors.len() {
            if self.pending & (1 << index) != 0 && !self.vectors[index].masked() {
                self.pending &= !(1 << index);
                self.fire(index);
            }
        }
    }

    fn fire(&mut self, index: usize) {
        let v = &mut self.vectors[index];
        let message = (v.address, v.data);
        if v.routed != Some(message) {
            if let Err(e) = self.platform.set_msi_route(v.gsi, v.address, v.data) {
                tracing::error!(error = %e, gsi = v.gsi, "cannot route an MSI-X vector");
                return;
            }
            v.routed = Some(message);
        }
        if let Err(e) = v.irq.write(1) {
            tracing::error!(error = %e, "cannot signal an MSI-X vector");
        }
    }

    #[cfg(test)]
    pub(crate) fn irq(&self, vector: u16) -> &EventFd {
        &self.vectors[usize::from(vector)].irq
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pci::Ids;
    use crate::pci::tests::FakePlatform;

    fn setup() -> (Msix, Arc<FakePlatform>, ConfigSpace, usize) {
        let platform = Arc::new(FakePlatform::default());
        let msix = Msix::new(2, platform.clone()).unwrap();
        let mut config = ConfigSpace::new(Ids {
            vendor: 1,
            device: 2,
            revision: 0,
            class: 0,
            subsystem_vendor: 0,
            subsystem: 0,
        });
        let cap = msix.add_capability(&mut config, 0, 0x4000, 0x5000);
        (msix, platform, config, cap)
    }

    fn program(msix: &mut Msix, vector: u64, address: u64, data: u32, masked: bool) {
        let base = vector * ENTRY_SIZE;
        msix.table_write(base, &(address as u32).to_le_bytes());
        msix.table_write(base + 4, &((address >> 32) as u32).to_le_bytes());
        msix.table_write(base + 8, &data.to_le_bytes());
        msix.table_write(base + 12, &u32::from(masked).to_le_bytes());
    }

    fn enable(msix: &mut Msix, config: &mut ConfigSpace, cap: usize, control: u16) {
        config.write(cap + 2, &control.to_le_bytes());
        msix.update_control(config, cap);
    }

    #[test]
    fn capability_layout() {
        let (_, _, config, cap) = setup();
        let mut b = [0u8; 12];
        config.read(cap, &mut b);
        assert_eq!(b[0], CAPABILITY_ID);
        assert_eq!(u16::from_le_bytes([b[2], b[3]]), 1, "table size - 1");
        assert_eq!(u32::from_le_bytes(b[4..8].try_into().unwrap()), 0x4000);
        assert_eq!(u32::from_le_bytes(b[8..12].try_into().unwrap()), 0x5000);
    }

    #[test]
    fn programmed_vectors_route_and_fire() {
        let (mut msix, platform, mut config, cap) = setup();
        program(&mut msix, 1, 0xfee0_1000, 0x4041, false);
        let mut entry = [0u8; 16];
        msix.table_read(16, &mut entry);
        assert_eq!(u32::from_le_bytes(entry[8..12].try_into().unwrap()), 0x4041);

        msix.notify(1);
        assert!(platform.routes.lock().unwrap().is_empty(), "MSI-X disabled");
        enable(&mut msix, &mut config, cap, CONTROL_ENABLE);
        msix.notify(1);
        msix.notify(1);
        assert_eq!(
            *platform.routes.lock().unwrap(),
            vec![(25, 0xfee0_1000, 0x4041)],
            "routed once for an unchanged message"
        );
        assert_eq!(msix.irq(1).read().unwrap(), 2);
        msix.notify(NO_VECTOR);
    }

    #[test]
    fn masked_vectors_are_pending_until_unmasked() {
        let (mut msix, _, mut config, cap) = setup();
        enable(&mut msix, &mut config, cap, CONTROL_ENABLE);
        program(&mut msix, 0, 0xfee0_0000, 0x30, true);
        msix.notify(0);
        let mut pba = [0u8; 8];
        msix.pba_read(0, &mut pba);
        assert_eq!(pba[0], 1, "pending while masked");
        msix.table_write(12, &0u32.to_le_bytes());
        assert_eq!(msix.irq(0).read().unwrap(), 1, "delivered on unmask");
        msix.pba_read(0, &mut pba);
        assert_eq!(pba[0], 0);

        // The function mask holds every vector back, too.
        enable(
            &mut msix,
            &mut config,
            cap,
            CONTROL_ENABLE | CONTROL_FUNCTION_MASK,
        );
        msix.notify(0);
        enable(&mut msix, &mut config, cap, CONTROL_ENABLE);
        assert_eq!(msix.irq(0).read().unwrap(), 1);
    }
}
