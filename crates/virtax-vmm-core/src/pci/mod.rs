//! A PCI Express root complex with one segment and one bus: type-0 configuration space reached
//! through ECAM, 32-bit memory BARs allocated from a fixed window, and the host bridge at 00:00.0.
//!
//! Devices implement [`PciFunction`]. The bus decodes ECAM and BAR accesses from vCPU MMIO exits and
//! tells a device when the guest moves one of its BARs or toggles memory decoding, so the device can
//! move its ioeventfds.

pub mod msix;

use std::io;

use vmm_sys_util::eventfd::EventFd;

use crate::layout::{PCI_ECAM_BASE, PCI_ECAM_SIZE, PCI_MMIO_BASE, PCI_MMIO_END};

pub const CONFIG_SIZE: usize = 256;
const VENDOR_ID: usize = 0x00;
const COMMAND: usize = 0x04;
const STATUS: usize = 0x06;
const REVISION: usize = 0x08;
const HEADER_TYPE: usize = 0x0e;
const BAR0: usize = 0x10;
const SUBSYSTEM_VENDOR_ID: usize = 0x2c;
const CAPABILITIES_POINTER: usize = 0x34;
const INTERRUPT_LINE: usize = 0x3c;
/// First byte after the type-0 header, where capabilities start.
const CAPABILITIES_START: usize = 0x40;

/// Command register: memory space decoding.
pub const COMMAND_MEMORY: u16 = 1 << 1;
/// Command register bits a guest may change: memory decoding, bus mastering, INTx disable.
const COMMAND_WRITABLE: u16 = COMMAND_MEMORY | (1 << 2) | (1 << 10);
const STATUS_CAPABILITIES: u16 = 1 << 4;
pub const BARS: usize = 6;
/// Slots on bus 0.
pub const SLOTS: usize = 32;

/// What the VMM needs from the hypervisor to wire a PCI device's interrupts and notifications.
pub trait Platform: Send + Sync {
    /// A GSI for one MSI-X vector, not used by anything else.
    fn allocate_gsi(&self) -> u32;
    fn register_irqfd(&self, event: &EventFd, gsi: u32) -> io::Result<()>;
    fn set_msi_route(&self, gsi: u32, address: u64, data: u32) -> io::Result<()>;
    fn register_mmio_ioevent(&self, event: &EventFd, addr: u64) -> io::Result<()>;
    fn unregister_mmio_ioevent(&self, event: &EventFd, addr: u64) -> io::Result<()>;
}

/// The identity of a PCI function.
#[derive(Debug, Clone, Copy)]
pub struct Ids {
    pub vendor: u16,
    pub device: u16,
    pub revision: u8,
    /// Class code, subclass and programming interface: `0xCCSSPP`.
    pub class: u32,
    pub subsystem_vendor: u16,
    pub subsystem: u16,
}

/// A type-0 configuration space: the registers, which bits the guest may write, and the BAR sizes.
pub struct ConfigSpace {
    regs: [u8; CONFIG_SIZE],
    wmask: [u8; CONFIG_SIZE],
    bar_sizes: [u32; BARS],
    next_capability: usize,
    /// Offset of the byte that points at the next capability (the header's pointer at first).
    last_link: usize,
}

impl ConfigSpace {
    pub fn new(ids: Ids) -> Self {
        let mut c = Self {
            regs: [0; CONFIG_SIZE],
            wmask: [0; CONFIG_SIZE],
            bar_sizes: [0; BARS],
            next_capability: CAPABILITIES_START,
            last_link: CAPABILITIES_POINTER,
        };
        c.set_u16(VENDOR_ID, ids.vendor);
        c.set_u16(VENDOR_ID + 2, ids.device);
        c.regs[REVISION] = ids.revision;
        c.regs[REVISION + 1..REVISION + 4].copy_from_slice(&ids.class.to_le_bytes()[..3]);
        c.regs[HEADER_TYPE] = 0;
        c.set_u16(SUBSYSTEM_VENDOR_ID, ids.subsystem_vendor);
        c.set_u16(SUBSYSTEM_VENDOR_ID + 2, ids.subsystem);
        c.wmask[COMMAND..COMMAND + 2].copy_from_slice(&COMMAND_WRITABLE.to_le_bytes());
        c.wmask[INTERRUPT_LINE] = 0xff;
        c
    }

    fn set_u16(&mut self, offset: usize, v: u16) {
        self.regs[offset..offset + 2].copy_from_slice(&v.to_le_bytes());
    }

    pub fn u16(&self, offset: usize) -> u16 {
        u16::from_le_bytes([self.regs[offset], self.regs[offset + 1]])
    }

    pub fn u32(&self, offset: usize) -> u32 {
        u32::from_le_bytes(self.regs[offset..offset + 4].try_into().unwrap())
    }

    /// Declare BAR `index` as a 32-bit, non-prefetchable memory BAR of `size` bytes (a power of two).
    pub fn add_bar(&mut self, index: usize, size: u32) {
        assert!(size.is_power_of_two() && size >= 16, "BAR size {size:#x}");
        self.bar_sizes[index] = size;
        let offset = BAR0 + 4 * index;
        self.wmask[offset..offset + 4].copy_from_slice(&(!(size - 1)).to_le_bytes());
    }

    pub fn bar_size(&self, index: usize) -> u32 {
        self.bar_sizes[index]
    }

    /// Program a BAR as firmware would before the guest starts.
    pub fn set_bar_address(&mut self, index: usize, addr: u32) {
        let offset = BAR0 + 4 * index;
        self.regs[offset..offset + 4].copy_from_slice(&addr.to_le_bytes());
    }

    /// The address BAR `index` decodes at, or `None` when it is absent, unprogrammed or memory
    /// decoding is off.
    pub fn bar_address(&self, index: usize) -> Option<u64> {
        let size = self.bar_sizes[index];
        if size == 0 || self.u16(COMMAND) & COMMAND_MEMORY == 0 {
            return None;
        }
        let addr = self.u32(BAR0 + 4 * index) & !0xf;
        // Zero is unprogrammed; all-ones-masked is the value left by BAR sizing.
        (addr != 0 && addr != !(size - 1)).then_some(u64::from(addr))
    }

    /// Append a capability with ID `id`; `body` follows the ID and next-pointer bytes, and
    /// `writable` (same length as `body`, or empty) marks the bits the guest may change.
    /// Returns the capability's offset.
    pub fn add_capability(&mut self, id: u8, body: &[u8], writable: &[u8]) -> usize {
        let offset = self.next_capability.next_multiple_of(4);
        let end = offset + 2 + body.len();
        assert!(
            end <= CONFIG_SIZE,
            "capabilities overflow configuration space"
        );
        self.regs[offset] = id;
        self.regs[offset + 2..end].copy_from_slice(body);
        if !writable.is_empty() {
            assert_eq!(writable.len(), body.len());
            self.wmask[offset + 2..end].copy_from_slice(writable);
        }
        self.regs[self.last_link] = offset as u8;
        self.last_link = offset + 1;
        self.next_capability = end;
        let status = self.u16(STATUS) | STATUS_CAPABILITIES;
        self.set_u16(STATUS, status);
        offset
    }

    /// Read `data.len()` bytes at `offset`; the PCIe extended space beyond 256 bytes reads as zero.
    pub fn read(&self, offset: usize, data: &mut [u8]) {
        for (i, b) in data.iter_mut().enumerate() {
            *b = self.regs.get(offset + i).copied().unwrap_or(0);
        }
    }

    /// Write the bits the guest may change; everything else keeps its value.
    pub fn write(&mut self, offset: usize, data: &[u8]) {
        for (i, b) in data.iter().enumerate() {
            let at = offset + i;
            if at < CONFIG_SIZE {
                self.regs[at] = (self.regs[at] & !self.wmask[at]) | (b & self.wmask[at]);
            }
        }
    }
}

/// A function on the PCI bus.
pub trait PciFunction: Send {
    fn config(&self) -> &ConfigSpace;
    fn config_mut(&mut self) -> &mut ConfigSpace;
    /// Called after the guest wrote `len` configuration bytes at `offset`.
    fn config_written(&mut self, _offset: usize, _len: usize) {}
    /// A guest read of `data.len()` bytes at `offset` inside BAR `bar`.
    fn bar_read(&mut self, _bar: usize, _offset: u64, data: &mut [u8]) {
        data.fill(0xff);
    }
    fn bar_write(&mut self, _bar: usize, _offset: u64, _data: &[u8]) {}
    /// BAR `bar` now decodes at `addr` (or nowhere, for `None`).
    fn bar_mapped(&mut self, _bar: usize, _addr: Option<u64>) {}
}

/// The host bridge at 00:00.0.
pub struct HostBridge(ConfigSpace);

impl HostBridge {
    pub fn new() -> Self {
        Self(ConfigSpace::new(Ids {
            // Red Hat's ID for a generic PCIe host bridge in a virtual machine.
            vendor: 0x1b36,
            device: 0x0008,
            revision: 0,
            class: 0x06_00_00,
            subsystem_vendor: 0x1b36,
            subsystem: 0x0008,
        }))
    }
}

impl Default for HostBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl PciFunction for HostBridge {
    fn config(&self) -> &ConfigSpace {
        &self.0
    }
    fn config_mut(&mut self) -> &mut ConfigSpace {
        &mut self.0
    }
}

struct Slot {
    function: Box<dyn PciFunction>,
    mapped: [Option<u64>; BARS],
}

/// Bus 0: up to 32 single-function devices.
pub struct PciBus {
    slots: Vec<Option<Slot>>,
    next_bar: u64,
}

impl Default for PciBus {
    fn default() -> Self {
        Self::new()
    }
}

impl PciBus {
    /// A bus holding only the host bridge.
    pub fn new() -> Self {
        let mut bus = Self {
            slots: (0..SLOTS).map(|_| None).collect(),
            next_bar: PCI_MMIO_BASE,
        };
        bus.add(Box::new(HostBridge::new()))
            .expect("an empty bus has room for the host bridge");
        bus
    }

    /// Put `function` in the next free slot and assign its BARs from the window, as firmware would.
    /// Returns the slot number.
    pub fn add(&mut self, mut function: Box<dyn PciFunction>) -> Result<u8, String> {
        let slot = self
            .slots
            .iter()
            .position(Option::is_none)
            .ok_or("every PCI slot on bus 0 is taken")?;
        for bar in 0..BARS {
            let size = u64::from(function.config().bar_size(bar));
            if size == 0 {
                continue;
            }
            let addr = self.next_bar.next_multiple_of(size);
            if addr + size > PCI_MMIO_END {
                return Err("the PCI memory window is full".into());
            }
            function.config_mut().set_bar_address(bar, addr as u32);
            self.next_bar = addr + size;
        }
        self.slots[slot] = Some(Slot {
            function,
            mapped: [None; BARS],
        });
        Ok(slot as u8)
    }

    /// Decode an ECAM offset into (slot, register); only function 0 of bus 0 exists.
    fn decode(offset: u64) -> Option<(usize, usize)> {
        let (bus, slot, function) = (offset >> 20, (offset >> 15) & 0x1f, (offset >> 12) & 0x7);
        (bus == 0 && function == 0).then_some((slot as usize, (offset & 0xfff) as usize))
    }

    pub fn ecam_read(&mut self, offset: u64, data: &mut [u8]) {
        match Self::decode(offset).and_then(|(s, reg)| Some((self.slots[s].as_ref()?, reg))) {
            Some((slot, reg)) => slot.function.config().read(reg, data),
            // No device: reads float high, so vendor ID 0xffff means "empty".
            None => data.fill(0xff),
        }
    }

    pub fn ecam_write(&mut self, offset: u64, data: &[u8]) {
        let Some((s, reg)) = Self::decode(offset) else {
            return;
        };
        let Some(slot) = self.slots[s].as_mut() else {
            return;
        };
        slot.function.config_mut().write(reg, data);
        slot.function.config_written(reg, data.len());
        for bar in 0..BARS {
            let now = slot.function.config().bar_address(bar);
            if now != slot.mapped[bar] {
                slot.mapped[bar] = now;
                slot.function.bar_mapped(bar, now);
            }
        }
    }

    /// Handle a guest MMIO read; `false` if no ECAM or BAR decodes `addr`.
    pub fn mmio_read(&mut self, addr: u64, data: &mut [u8]) -> bool {
        if (PCI_ECAM_BASE..PCI_ECAM_BASE + PCI_ECAM_SIZE).contains(&addr) {
            self.ecam_read(addr - PCI_ECAM_BASE, data);
            return true;
        }
        match self.bar_at(addr) {
            Some((slot, bar, offset)) => {
                self.slots[slot]
                    .as_mut()
                    .expect("bar_at returns occupied slots")
                    .function
                    .bar_read(bar, offset, data);
                true
            }
            None => false,
        }
    }

    /// Handle a guest MMIO write; `false` if no ECAM or BAR decodes `addr`.
    pub fn mmio_write(&mut self, addr: u64, data: &[u8]) -> bool {
        if (PCI_ECAM_BASE..PCI_ECAM_BASE + PCI_ECAM_SIZE).contains(&addr) {
            self.ecam_write(addr - PCI_ECAM_BASE, data);
            return true;
        }
        match self.bar_at(addr) {
            Some((slot, bar, offset)) => {
                self.slots[slot]
                    .as_mut()
                    .expect("bar_at returns occupied slots")
                    .function
                    .bar_write(bar, offset, data);
                true
            }
            None => false,
        }
    }

    /// The slot, BAR and offset that decode `addr`.
    fn bar_at(&self, addr: u64) -> Option<(usize, usize, u64)> {
        self.slots.iter().enumerate().find_map(|(i, slot)| {
            let slot = slot.as_ref()?;
            (0..BARS).find_map(|bar| {
                let base = slot.mapped[bar]?;
                let size = u64::from(slot.function.config().bar_size(bar));
                (base..base + size)
                    .contains(&addr)
                    .then_some((i, bar, addr - base))
            })
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Records what devices ask of the hypervisor.
    #[derive(Default)]
    pub struct FakePlatform {
        next_gsi: AtomicU32,
        pub routes: Mutex<Vec<(u32, u64, u32)>>,
        pub ioevents: Mutex<Vec<u64>>,
    }

    impl Platform for FakePlatform {
        fn allocate_gsi(&self) -> u32 {
            24 + self.next_gsi.fetch_add(1, Ordering::Relaxed)
        }
        fn register_irqfd(&self, _: &EventFd, _: u32) -> io::Result<()> {
            Ok(())
        }
        fn set_msi_route(&self, gsi: u32, address: u64, data: u32) -> io::Result<()> {
            self.routes.lock().unwrap().push((gsi, address, data));
            Ok(())
        }
        fn register_mmio_ioevent(&self, _: &EventFd, addr: u64) -> io::Result<()> {
            self.ioevents.lock().unwrap().push(addr);
            Ok(())
        }
        fn unregister_mmio_ioevent(&self, _: &EventFd, addr: u64) -> io::Result<()> {
            self.ioevents.lock().unwrap().retain(|a| *a != addr);
            Ok(())
        }
    }

    struct TestDevice {
        config: ConfigSpace,
        mapped: Vec<Option<u64>>,
        last_write: Option<(u64, Vec<u8>)>,
    }

    impl PciFunction for TestDevice {
        fn config(&self) -> &ConfigSpace {
            &self.config
        }
        fn config_mut(&mut self) -> &mut ConfigSpace {
            &mut self.config
        }
        fn bar_read(&mut self, _: usize, offset: u64, data: &mut [u8]) {
            data.fill(offset as u8);
        }
        fn bar_write(&mut self, _: usize, offset: u64, data: &[u8]) {
            self.last_write = Some((offset, data.to_vec()));
        }
        fn bar_mapped(&mut self, _: usize, addr: Option<u64>) {
            self.mapped.push(addr);
        }
    }

    fn test_device() -> TestDevice {
        let mut config = ConfigSpace::new(Ids {
            vendor: 0x1af4,
            device: 0x1042,
            revision: 1,
            class: 0x01_80_00,
            subsystem_vendor: 0x1af4,
            subsystem: 0x40,
        });
        config.add_bar(0, 0x4000);
        TestDevice {
            config,
            mapped: Vec::new(),
            last_write: None,
        }
    }

    fn ecam(slot: u64, reg: u64) -> u64 {
        PCI_ECAM_BASE + (slot << 15) + reg
    }

    fn read32(bus: &mut PciBus, addr: u64) -> u32 {
        let mut d = [0u8; 4];
        assert!(bus.mmio_read(addr, &mut d));
        u32::from_le_bytes(d)
    }

    #[test]
    fn ecam_decodes_slots_and_empty_slots_float_high() {
        let mut bus = PciBus::new();
        assert_eq!(read32(&mut bus, ecam(0, 0)), 0x0008_1b36, "host bridge");
        assert_eq!(read32(&mut bus, ecam(0, 8)) >> 8, 0x06_00_00, "class");
        assert_eq!(read32(&mut bus, ecam(1, 0)), 0xffff_ffff, "empty slot");
        // Function 1 of slot 0 does not exist.
        assert_eq!(read32(&mut bus, ecam(0, 0) + (1 << 12)), 0xffff_ffff);
        // Outside ECAM and with no BARs mapped, nothing decodes.
        assert!(!bus.mmio_read(PCI_MMIO_BASE, &mut [0u8; 4]));
    }

    #[test]
    fn config_writes_respect_the_write_mask() {
        let mut bus = PciBus::new();
        let slot = u64::from(bus.add(Box::new(test_device())).unwrap());
        bus.mmio_write(ecam(slot, 0), &0u32.to_le_bytes());
        assert_eq!(
            read32(&mut bus, ecam(slot, 0)),
            0x1042_1af4,
            "IDs are read-only"
        );
        bus.mmio_write(ecam(slot, 4), &0xffffu16.to_le_bytes());
        assert_eq!(
            read32(&mut bus, ecam(slot, 4)) & 0xffff,
            u32::from(COMMAND_WRITABLE)
        );
    }

    #[test]
    fn bar_sizing_assignment_and_mapping() {
        let mut bus = PciBus::new();
        let slot = u64::from(bus.add(Box::new(test_device())).unwrap());
        let bar0 = ecam(slot, BAR0 as u64);
        assert_eq!(
            u64::from(read32(&mut bus, bar0)),
            PCI_MMIO_BASE,
            "assigned like firmware"
        );
        // Sizing: write all ones, read back the size mask.
        bus.mmio_write(bar0, &u32::MAX.to_le_bytes());
        assert_eq!(read32(&mut bus, bar0), !(0x4000 - 1));
        assert!(
            !bus.mmio_read(PCI_MMIO_BASE, &mut [0u8; 4]),
            "decoding is off"
        );
        // Program an address and enable memory decoding: the BAR decodes there.
        let addr = PCI_MMIO_BASE + 0x10_0000;
        bus.mmio_write(bar0, &(addr as u32).to_le_bytes());
        bus.mmio_write(ecam(slot, 4), &COMMAND_MEMORY.to_le_bytes());
        let mut d = [0u8; 2];
        assert!(bus.mmio_read(addr + 0x12, &mut d));
        assert_eq!(d, [0x12, 0x12], "offset inside the BAR");
        assert!(bus.mmio_write(addr + 0x20, &[7]));
        assert!(!bus.mmio_read(addr + 0x4000, &mut d), "past the BAR's end");
        // Turning decoding off unmaps it again.
        bus.mmio_write(ecam(slot, 4), &0u16.to_le_bytes());
        assert!(!bus.mmio_read(addr, &mut d));
    }

    #[test]
    fn capabilities_are_chained() {
        let mut d = test_device();
        let first = d.config.add_capability(0x09, &[1, 2], &[]);
        let second = d.config.add_capability(0x11, &[0; 10], &[]);
        assert_eq!(first, CAPABILITIES_START);
        assert_eq!(d.config.regs[CAPABILITIES_POINTER] as usize, first);
        assert_eq!(d.config.regs[first + 1] as usize, second);
        assert_eq!(d.config.regs[second + 1], 0, "end of the list");
        assert_ne!(d.config.u16(STATUS) & STATUS_CAPABILITIES, 0);
    }
}
