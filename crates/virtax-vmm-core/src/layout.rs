//! Guest-physical memory layout for x86_64 direct kernel boot.

use vm_memory::{GuestAddress, GuestMemoryMmap};

/// Boot GDT and (empty) IDT.
pub const GDT_ADDR: u64 = 0x500;
pub const IDT_ADDR: u64 = 0x520;
/// Linux `boot_params` ("zero page").
pub const ZERO_PAGE: u64 = 0x7000;
/// Initial stack pointer for the boot vCPU.
pub const BOOT_STACK: u64 = 0x8ff0;
/// Identity-mapping page tables for the first GiB: PML4, PDPT, PD.
pub const PML4_ADDR: u64 = 0x9000;
pub const PDPT_ADDR: u64 = 0xa000;
pub const PD_ADDR: u64 = 0xb000;
/// Kernel command line.
pub const CMDLINE_ADDR: u64 = 0x20000;
/// Start of the Extended BIOS Data Area; RAM below it is usable.
pub const EBDA_START: u64 = 0x9fc00;
/// Kernels are loaded at 1 MiB.
pub const HIMEM_START: u64 = 0x10_0000;
/// The 32-bit MMIO hole: [3 GiB, 4 GiB) is never RAM.
pub const MMIO_GAP_START: u64 = 0xc000_0000;
pub const MMIO_GAP_END: u64 = 0x1_0000_0000;

pub const MIB: u64 = 1 << 20;
/// Smallest guest the boot path supports.
pub const MIN_MEMORY_MIB: u64 = 64;

// The boot stack grows down between the zero page and the page tables, and the largest command
// line the kernel accepts fits below the EBDA. Checked at compile time.
const _: () = assert!(BOOT_STACK < PML4_ADDR && BOOT_STACK > ZERO_PAGE + 4096);
const _: () = assert!(CMDLINE_ADDR + 0x10000 <= EBDA_START);

/// RAM ranges for `memory_mib`, split around the MMIO gap.
pub fn ram_ranges(memory_mib: u64) -> Vec<(GuestAddress, usize)> {
    let total = memory_mib * MIB;
    if total <= MMIO_GAP_START {
        vec![(GuestAddress(0), total as usize)]
    } else {
        vec![
            (GuestAddress(0), MMIO_GAP_START as usize),
            (
                GuestAddress(MMIO_GAP_END),
                (total - MMIO_GAP_START) as usize,
            ),
        ]
    }
}

pub fn guest_memory(memory_mib: u64) -> Result<GuestMemoryMmap, String> {
    GuestMemoryMmap::from_ranges(&ram_ranges(memory_mib))
        .map_err(|e| format!("cannot allocate guest memory: {e}"))
}

/// End (exclusive) of RAM below the MMIO gap.
pub fn low_ram_end(memory_mib: u64) -> u64 {
    (memory_mib * MIB).min(MMIO_GAP_START)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_skip_the_mmio_gap() {
        assert_eq!(ram_ranges(512), vec![(GuestAddress(0), 512 << 20)]);
        assert_eq!(ram_ranges(3072), vec![(GuestAddress(0), 3 << 30)]);
        assert_eq!(
            ram_ranges(4096),
            vec![
                (GuestAddress(0), 3 << 30),
                (GuestAddress(MMIO_GAP_END), 1 << 30)
            ]
        );
        assert_eq!(low_ram_end(4096), MMIO_GAP_START);
    }

    #[test]
    fn boot_structures_do_not_overlap() {
        let mut areas = [
            (GDT_ADDR, 4 * 8),
            (IDT_ADDR, 8),
            (ZERO_PAGE, 4096),
            (PML4_ADDR, 4096),
            (PDPT_ADDR, 4096),
            (PD_ADDR, 4096),
            (CMDLINE_ADDR, 4096),
        ];
        areas.sort();
        for w in areas.windows(2) {
            assert!(
                w[0].0 + w[0].1 <= w[1].0,
                "{:#x} overlaps {:#x}",
                w[0].0,
                w[1].0
            );
        }
    }
}
