//! Load a Linux kernel (bzImage or ELF vmlinux), initramfs and command line, and fill `boot_params`
//! following the x86 boot protocol (Documentation/arch/x86/boot.rst).

use std::fs::File;
use std::path::Path;

use linux_loader::cmdline::Cmdline;
use linux_loader::loader::bootparam::{boot_e820_entry, boot_params, setup_header};
use linux_loader::loader::bzimage::BzImage;
use linux_loader::loader::elf::Elf;
use linux_loader::loader::{KernelLoader, load_cmdline};
use vm_memory::{Bytes, GuestAddress, GuestMemoryMmap};

use crate::layout::{CMDLINE_ADDR, EBDA_START, HIMEM_START, MMIO_GAP_END, ZERO_PAGE, low_ram_end};

const E820_RAM: u32 = 1;
const KERNEL_BOOT_FLAG_MAGIC: u16 = 0xaa55;
const KERNEL_HDR_MAGIC: u32 = 0x5372_6448; // "HdrS"
const KERNEL_LOADER_OTHER: u8 = 0xff;
const KERNEL_MIN_ALIGNMENT: u32 = 0x0100_0000;
/// The 64-bit entry point of a bzImage is 0x200 bytes into its protected-mode code.
const BZIMAGE_64BIT_ENTRY_OFFSET: u64 = 0x200;
const PAGE: u64 = 4096;

/// Where the boot vCPU starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootEntry {
    pub rip: u64,
    pub zero_page: u64,
}

pub struct BootImages<'a> {
    pub kernel: &'a Path,
    pub initrd: Option<&'a Path>,
    pub cmdline: &'a str,
    pub memory_mib: u64,
    /// Guest-physical address of the ACPI RSDP, if the VM has ACPI tables.
    pub acpi_rsdp: Option<u64>,
}

fn open(path: &Path, what: &str) -> Result<File, String> {
    File::open(path).map_err(|e| format!("cannot open {what} {}: {e}", path.display()))
}

/// Load everything into guest memory and write the zero page.
pub fn load(mem: &GuestMemoryMmap, images: &BootImages<'_>) -> Result<BootEntry, String> {
    let mut kernel = open(images.kernel, "kernel")?;
    let (rip, kernel_end, mut hdr) =
        match BzImage::load(mem, None, &mut kernel, Some(GuestAddress(HIMEM_START))) {
            Ok(r) => {
                let hdr = r.setup_header.ok_or("bzImage without a setup header")?;
                (
                    r.kernel_load.0 + BZIMAGE_64BIT_ENTRY_OFFSET,
                    r.kernel_end,
                    hdr,
                )
            }
            Err(bz_err) => {
                let mut kernel = open(images.kernel, "kernel")?;
                let r = Elf::load(mem, None, &mut kernel, Some(GuestAddress(HIMEM_START)))
                    .map_err(|elf_err| {
                        format!(
                            "{} is neither a bzImage ({bz_err}) nor an ELF vmlinux ({elf_err})",
                            images.kernel.display()
                        )
                    })?;
                let hdr = setup_header {
                    boot_flag: KERNEL_BOOT_FLAG_MAGIC,
                    header: KERNEL_HDR_MAGIC,
                    kernel_alignment: KERNEL_MIN_ALIGNMENT,
                    ..Default::default()
                };
                (r.kernel_load.0, r.kernel_end, hdr)
            }
        };
    hdr.type_of_loader = KERNEL_LOADER_OTHER;

    // Command line.
    let capacity = if hdr.cmdline_size > 0 {
        hdr.cmdline_size as usize + 1
    } else {
        4096
    };
    let mut cmdline = Cmdline::new(capacity).map_err(|e| format!("command line: {e}"))?;
    cmdline
        .insert_str(images.cmdline)
        .map_err(|e| format!("command line: {e}"))?;
    load_cmdline(mem, GuestAddress(CMDLINE_ADDR), &cmdline)
        .map_err(|e| format!("command line: {e}"))?;
    hdr.cmd_line_ptr = CMDLINE_ADDR as u32;
    hdr.cmdline_size = images.cmdline.len() as u32 + 1;

    // Initramfs: as high as allowed in low RAM, above the kernel.
    if let Some(path) = images.initrd {
        let data = std::fs::read(path)
            .map_err(|e| format!("cannot read initrd {}: {e}", path.display()))?;
        let max = if hdr.initrd_addr_max > 0 {
            u64::from(hdr.initrd_addr_max) + 1
        } else {
            1 << 32
        };
        let addr = initrd_address(
            low_ram_end(images.memory_mib).min(max),
            data.len() as u64,
            kernel_end,
        )?;
        mem.write_slice(&data, GuestAddress(addr))
            .map_err(|e| format!("initrd: {e}"))?;
        hdr.ramdisk_image = addr as u32;
        hdr.ramdisk_size = data.len() as u32;
    }

    let mut params = boot_params {
        hdr,
        ..Default::default()
    };
    params.acpi_rsdp_addr = images.acpi_rsdp.unwrap_or(0);
    let e820 = e820_map(images.memory_mib);
    params.e820_entries = e820.len() as u8;
    params.e820_table[..e820.len()].copy_from_slice(&e820);
    mem.write_obj(params, GuestAddress(ZERO_PAGE))
        .map_err(|e| format!("zero page: {e}"))?;
    Ok(BootEntry {
        rip,
        zero_page: ZERO_PAGE,
    })
}

/// Page-aligned address for an initrd of `size` bytes ending at or below `limit`, above `kernel_end`.
pub fn initrd_address(limit: u64, size: u64, kernel_end: u64) -> Result<u64, String> {
    let addr = limit.checked_sub(size).map(|a| a & !(PAGE - 1));
    match addr {
        Some(a) if a >= kernel_end => Ok(a),
        _ => Err(format!(
            "initrd of {size} bytes does not fit between the kernel and {limit:#x}; add guest memory"
        )),
    }
}

/// The e820 memory map handed to the guest.
pub fn e820_map(memory_mib: u64) -> Vec<boot_e820_entry> {
    let ram = |addr: u64, end: u64| boot_e820_entry {
        addr,
        size: end - addr,
        r#type: E820_RAM,
    };
    let total = memory_mib * crate::layout::MIB;
    let low_end = low_ram_end(memory_mib);
    let mut map = vec![ram(0, EBDA_START), ram(HIMEM_START, low_end)];
    if total > low_end {
        map.push(ram(MMIO_GAP_END, MMIO_GAP_END + (total - low_end)));
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e820_covers_ram_and_skips_the_holes() {
        let small: Vec<_> = e820_map(256).iter().map(|e| (e.addr, e.size)).collect();
        assert_eq!(
            small,
            vec![(0, EBDA_START), (HIMEM_START, (256 << 20) - HIMEM_START)]
        );
        let big = e820_map(5120);
        assert_eq!(big.len(), 3);
        assert_eq!((big[2].addr, big[2].size), (MMIO_GAP_END, 2 << 30));
    }

    #[test]
    fn initrd_goes_high_and_page_aligned() {
        assert_eq!(
            initrd_address(0x1000_0000, 0x1234, 0x200_0000).unwrap(),
            0xfffe000
        );
        assert!(
            initrd_address(0x300_0000, 0x200_0000, 0x200_0000).is_err(),
            "would overlap the kernel"
        );
    }

    #[test]
    fn rejects_a_file_that_is_not_a_kernel() {
        let mem = crate::layout::guest_memory(64).unwrap();
        let dir = std::env::temp_dir().join(format!("virtax-vmm-notkernel-{}", std::process::id()));
        std::fs::write(&dir, b"definitely not a kernel").unwrap();
        let images = BootImages {
            kernel: &dir,
            initrd: None,
            cmdline: "console=ttyS0",
            memory_mib: 64,
            acpi_rsdp: None,
        };
        let err = load(&mem, &images).unwrap_err();
        std::fs::remove_file(&dir).unwrap();
        assert!(err.contains("neither a bzImage"), "{err}");
    }
}
