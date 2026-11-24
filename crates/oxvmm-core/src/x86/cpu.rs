//! Initial state of the boot vCPU: 64-bit long mode with identity-mapped low memory, as the Linux
//! 64-bit boot protocol expects.

use oxvmm_hv::kvm::kvm_bindings::{
    CpuId, kvm_fpu, kvm_msr_entry, kvm_regs, kvm_segment, kvm_sregs,
};
use oxvmm_hv::kvm::{HvError, Vcpu};
use vm_memory::{Bytes, GuestAddress, GuestMemoryMmap};

use super::boot::BootEntry;
use crate::layout::{BOOT_STACK, GDT_ADDR, IDT_ADDR, PD_ADDR, PDPT_ADDR, PML4_ADDR};

const X86_CR0_PE: u64 = 0x1;
const X86_CR0_PG: u64 = 0x8000_0000;
const X86_CR4_PAE: u64 = 0x20;
const EFER_LME: u64 = 0x100;
const EFER_LMA: u64 = 0x400;

/// Boot GDT: null, 64-bit code, data, TSS.
pub const BOOT_GDT: [u64; 4] = [
    gdt_entry(0, 0, 0),
    gdt_entry(0xa09b, 0, 0xfffff),
    gdt_entry(0xc093, 0, 0xfffff),
    gdt_entry(0x808b, 0, 0xfffff),
];

/// Encode a segment descriptor.
pub const fn gdt_entry(flags: u16, base: u32, limit: u32) -> u64 {
    ((base as u64 & 0xff00_0000) << (56 - 24))
        | ((flags as u64 & 0x0000_f0ff) << 40)
        | ((limit as u64 & 0x000f_0000) << (48 - 16))
        | ((base as u64 & 0x00ff_ffff) << 16)
        | (limit as u64 & 0x0000_ffff)
}

/// Decode a descriptor into KVM's segment register form.
pub fn segment(entry: u64, index: u16) -> kvm_segment {
    let bit = |n: u32| ((entry >> n) & 1) as u8;
    let base = ((entry & 0xff00_0000_0000_0000) >> 32)
        | ((entry & 0x0000_00ff_0000_0000) >> 16)
        | ((entry & 0x0000_0000_ffff_0000) >> 16);
    let limit = ((entry & 0x000f_0000_0000_0000) >> 32) | (entry & 0xffff);
    let granular = bit(55);
    kvm_segment {
        base,
        limit: if granular == 1 {
            ((limit << 12) | 0xfff) as u32
        } else {
            limit as u32
        },
        selector: index * 8,
        type_: ((entry >> 40) & 0xf) as u8,
        present: bit(47),
        dpl: ((entry >> 45) & 0x3) as u8,
        db: bit(54),
        s: bit(44),
        l: bit(53),
        g: granular,
        avl: bit(52),
        unusable: if bit(47) == 1 { 0 } else { 1 },
        padding: 0,
    }
}

/// Write the GDT, an empty IDT and page tables identity-mapping the first GiB with 2 MiB pages.
pub fn write_boot_tables(mem: &GuestMemoryMmap) -> Result<(), String> {
    let err = |e: vm_memory::GuestMemoryError| format!("boot tables: {e}");
    for (i, entry) in BOOT_GDT.iter().enumerate() {
        mem.write_obj(*entry, GuestAddress(GDT_ADDR + 8 * i as u64))
            .map_err(err)?;
    }
    mem.write_obj(0u64, GuestAddress(IDT_ADDR)).map_err(err)?;
    mem.write_obj(PDPT_ADDR | 0x03, GuestAddress(PML4_ADDR))
        .map_err(err)?;
    mem.write_obj(PD_ADDR | 0x03, GuestAddress(PDPT_ADDR))
        .map_err(err)?;
    for i in 0..512u64 {
        // Present | writable | 2 MiB page.
        mem.write_obj((i << 21) | 0x83, GuestAddress(PD_ADDR + i * 8))
            .map_err(err)?;
    }
    Ok(())
}

fn long_mode_sregs(mut sregs: kvm_sregs) -> kvm_sregs {
    let code = segment(BOOT_GDT[1], 1);
    let data = segment(BOOT_GDT[2], 2);
    sregs.gdt.base = GDT_ADDR;
    sregs.gdt.limit = (std::mem::size_of_val(&BOOT_GDT) - 1) as u16;
    sregs.idt.base = IDT_ADDR;
    sregs.idt.limit = 7;
    sregs.cs = code;
    sregs.ds = data;
    sregs.es = data;
    sregs.fs = data;
    sregs.gs = data;
    sregs.ss = data;
    sregs.tr = segment(BOOT_GDT[3], 3);
    sregs.cr3 = PML4_ADDR;
    sregs.cr4 |= X86_CR4_PAE;
    sregs.cr0 |= X86_CR0_PE | X86_CR0_PG;
    sregs.efer |= EFER_LME | EFER_LMA;
    sregs
}

/// MSRs Linux expects at boot: zeroed syscall/sysenter state, fast strings, write-back MTRR default.
fn boot_msrs() -> Vec<kvm_msr_entry> {
    let msr = |index: u32, data: u64| kvm_msr_entry {
        index,
        data,
        ..Default::default()
    };
    vec![
        msr(0x174, 0),             // IA32_SYSENTER_CS
        msr(0x175, 0),             // IA32_SYSENTER_ESP
        msr(0x176, 0),             // IA32_SYSENTER_EIP
        msr(0xc000_0081, 0),       // STAR
        msr(0xc000_0082, 0),       // LSTAR
        msr(0xc000_0083, 0),       // CSTAR
        msr(0xc000_0084, 0),       // SYSCALL_MASK
        msr(0xc000_0102, 0),       // KERNEL_GS_BASE
        msr(0x10, 0),              // TSC
        msr(0x1a0, 1),             // IA32_MISC_ENABLE: fast strings
        msr(0x2ff, (1 << 11) | 6), // MTRR default type: enabled, write-back
    ]
}

/// Expose the host's KVM-supported CPUID, identifying vCPU `id` of `count` and a hypervisor.
pub fn guest_cpuid(mut cpuid: CpuId, id: u8, count: u8) -> CpuId {
    for entry in cpuid.as_mut_slice() {
        if entry.function == 1 {
            entry.ebx =
                (entry.ebx & 0x0000_ffff) | (u32::from(count) << 16) | (u32::from(id) << 24);
            entry.ecx |= 1 << 31; // hypervisor present
        }
    }
    cpuid
}

const APIC_LVT_LINT0: usize = 0x350;
const APIC_LVT_LINT1: usize = 0x360;
const APIC_MODE_NMI: u32 = 0x4;
const APIC_MODE_EXTINT: u32 = 0x7;

fn set_delivery_mode(regs: &mut [std::os::raw::c_char], offset: usize, mode: u32) {
    let mut bytes = [0u8; 4];
    for (b, r) in bytes.iter_mut().zip(&regs[offset..offset + 4]) {
        *b = *r as u8;
    }
    let value = (u32::from_le_bytes(bytes) & !0x700) | (mode << 8);
    for (r, b) in regs[offset..offset + 4].iter_mut().zip(value.to_le_bytes()) {
        *r = b as std::os::raw::c_char;
    }
}

/// Configure vCPU `id` of `count`. The boot vCPU also gets 64-bit mode at `entry`; the others wait
/// for the guest to start them with INIT/SIPI, which the in-kernel local APIC handles.
pub fn setup_vcpu(
    vcpu: &Vcpu,
    cpuid: CpuId,
    id: u8,
    count: u8,
    entry: Option<BootEntry>,
) -> Result<(), HvError> {
    vcpu.set_cpuid(&guest_cpuid(cpuid, id, count))?;
    vcpu.set_msrs(&boot_msrs())?;
    if let Some(entry) = entry {
        vcpu.set_sregs(&long_mode_sregs(vcpu.sregs()?))?;
        vcpu.set_regs(&kvm_regs {
            rflags: 0x2,
            rip: entry.rip,
            rsp: BOOT_STACK,
            rbp: BOOT_STACK,
            rsi: entry.zero_page,
            ..Default::default()
        })?;
        vcpu.set_fpu(&kvm_fpu {
            fcw: 0x37f,
            mxcsr: 0x1f80,
            ..Default::default()
        })?;
    }
    // LINT0 receives legacy PIC interrupts, LINT1 NMIs.
    let mut lapic = vcpu.lapic()?;
    set_delivery_mode(&mut lapic.regs, APIC_LVT_LINT0, APIC_MODE_EXTINT);
    set_delivery_mode(&mut lapic.regs, APIC_LVT_LINT1, APIC_MODE_NMI);
    vcpu.set_lapic(&lapic)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gdt_encoding_matches_reference_values() {
        assert_eq!(BOOT_GDT[0], 0);
        assert_eq!(BOOT_GDT[1], 0x00af_9b00_0000_ffff);
        assert_eq!(BOOT_GDT[2], 0x00cf_9300_0000_ffff);
        let code = segment(BOOT_GDT[1], 1);
        assert_eq!(
            (code.selector, code.l, code.present, code.type_, code.limit),
            (8, 1, 1, 0xb, 0xffff_ffff)
        );
        assert_eq!(segment(BOOT_GDT[0], 0).unusable, 1);
    }

    #[test]
    fn long_mode_registers() {
        let s = long_mode_sregs(kvm_sregs::default());
        assert_eq!(s.cr0 & (X86_CR0_PE | X86_CR0_PG), X86_CR0_PE | X86_CR0_PG);
        assert_eq!(s.efer & (EFER_LME | EFER_LMA), EFER_LME | EFER_LMA);
        assert_eq!((s.cr3, s.gdt.limit), (PML4_ADDR, 31));
    }

    #[test]
    fn page_tables_identity_map_the_first_gib() {
        let mem = crate::layout::guest_memory(64).unwrap();
        write_boot_tables(&mem).unwrap();
        let pml4: u64 = mem.read_obj(GuestAddress(PML4_ADDR)).unwrap();
        let pd_last: u64 = mem.read_obj(GuestAddress(PD_ADDR + 511 * 8)).unwrap();
        assert_eq!(pml4, PDPT_ADDR | 3);
        assert_eq!(pd_last, (511 << 21) | 0x83);
    }

    #[test]
    fn lapic_delivery_mode_is_patched_in_place() {
        let mut regs = [0 as std::os::raw::c_char; 1024];
        regs[APIC_LVT_LINT0 + 1] = 0x07; // stale mode bits
        regs[APIC_LVT_LINT0] = 0x10;
        set_delivery_mode(&mut regs, APIC_LVT_LINT0, APIC_MODE_NMI);
        assert_eq!(
            (regs[APIC_LVT_LINT0] as u8, regs[APIC_LVT_LINT0 + 1] as u8),
            (0x10, 0x04)
        );
    }
}
