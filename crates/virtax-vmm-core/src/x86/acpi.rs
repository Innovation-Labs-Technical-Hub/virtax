//! ACPI tables for a hardware-reduced platform: RSDP → XSDT → {FADT → DSDT, MADT}.
//!
//! The MADT lists every vCPU's local APIC and the I/O APIC, which is what lets Linux bring up more
//! than one CPU. The FADT is hardware-reduced (no PM timer, SCI or fixed events) and points its sleep
//! control register at [`SLEEP_CONTROL_PORT`]; together with the DSDT's `\_S5_` object that lets the
//! guest power itself off. The DSDT also describes COM1, the CMOS RTC and a power button raised through
//! a Generic Event Device. The FADT declares no PS/2 controller or VGA, so Linux does not probe hardware
//! virtax-vmm does not emulate.

use vm_memory::{Bytes, GuestAddress, GuestMemoryMmap};

/// Where the tables live: the BIOS area below 1 MiB, which the e820 map does not report as RAM.
pub const RSDP_ADDR: u64 = 0x000e_0000;
const XSDT_ADDR: u64 = RSDP_ADDR + 0x40;
const FADT_ADDR: u64 = RSDP_ADDR + 0x100;
const DSDT_ADDR: u64 = RSDP_ADDR + 0x300;
const MADT_ADDR: u64 = RSDP_ADDR + 0x400;

/// I/O ports of the ACPI sleep control/status and reset registers.
pub const SLEEP_CONTROL_PORT: u16 = 0x600;
pub const SLEEP_STATUS_PORT: u16 = 0x601;
pub const RESET_PORT: u16 = 0x602;
pub const RESET_VALUE: u8 = 0x1;
/// Sleep-control bits (ACPI 6.5 §4.8.3.7): SLP_EN and the SLP_TYP field.
pub const SLP_EN: u8 = 1 << 5;
pub const SLP_TYP_SHIFT: u8 = 2;
/// The SLP_TYP value `\_S5_` advertises for soft-off.
pub const S5_SLP_TYP: u8 = 5;

/// Interrupt of the Generic Event Device that delivers the power button.
pub const GED_GSI: u32 = 5;
/// ACPI notification value for "power button pressed" (ACPI 6.5 §5.6.6).
const NOTIFY_POWER_BUTTON: u8 = 0x80;

pub const LOCAL_APIC_ADDR: u32 = 0xfee0_0000;
pub const IO_APIC_ADDR: u32 = 0xfec0_0000;
/// Highest vCPU count the MADT and xAPIC IDs (8 bits, one taken by the I/O APIC) allow here.
pub const MAX_VCPUS: u8 = 254;

const OEM_ID: &[u8; 6] = b"VIRTAX";
const OEM_TABLE_ID: &[u8; 8] = b"VTXVMM  ";

fn checksum(bytes: &[u8]) -> u8 {
    0u8.wrapping_sub(bytes.iter().fold(0u8, |a, b| a.wrapping_add(*b)))
}

/// A table with a standard 36-byte header; `finish` fills in length and checksum.
struct Table(Vec<u8>);

impl Table {
    fn new(signature: &[u8; 4], revision: u8) -> Self {
        let mut b = Vec::with_capacity(256);
        b.extend_from_slice(signature);
        b.extend_from_slice(&0u32.to_le_bytes()); // length, patched in finish()
        b.push(revision);
        b.push(0); // checksum, patched in finish()
        b.extend_from_slice(OEM_ID);
        b.extend_from_slice(OEM_TABLE_ID);
        b.extend_from_slice(&1u32.to_le_bytes()); // OEM revision
        b.extend_from_slice(b"VRTX"); // creator ID
        b.extend_from_slice(&1u32.to_le_bytes()); // creator revision
        Self(b)
    }

    fn u8(&mut self, v: u8) -> &mut Self {
        self.0.push(v);
        self
    }
    fn u16(&mut self, v: u16) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    fn u32(&mut self, v: u32) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    fn u64(&mut self, v: u64) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    fn bytes(&mut self, v: &[u8]) -> &mut Self {
        self.0.extend_from_slice(v);
        self
    }
    fn zeros(&mut self, n: usize) -> &mut Self {
        self.0.resize(self.0.len() + n, 0);
        self
    }
    /// A Generic Address Structure for a byte-wide I/O port (or all zero for "absent").
    fn io_gas(&mut self, port: Option<u16>) -> &mut Self {
        match port {
            Some(p) => self.u8(1).u8(8).u8(0).u8(1).u64(u64::from(p)),
            None => self.zeros(12),
        }
    }

    fn finish(mut self) -> Vec<u8> {
        let len = self.0.len() as u32;
        self.0[4..8].copy_from_slice(&len.to_le_bytes());
        self.0[9] = checksum(&self.0);
        self.0
    }
}

/// The DSDT: `\_S5_` for soft-off, and COM1 described as a PNP0501 device. On a hardware-reduced
/// platform Linux only gives ISA devices an interrupt when the firmware describes them.
fn dsdt_aml() -> Vec<u8> {
    use super::aml;
    use crate::devices::{COM1_BASE, COM1_IRQ};
    let com1 = aml::device(
        "COM1",
        [
            aml::name("_HID", aml::eisa_id("PNP0501")),
            aml::name("_UID", aml::integer(0)),
            aml::name("_CRS", aml::io_irq_resources(COM1_BASE, 8, COM1_IRQ as u8)),
        ]
        .concat(),
    );
    let rtc = aml::device(
        "RTC_",
        [
            aml::name("_HID", aml::eisa_id("PNP0B00")),
            aml::name("_CRS", aml::io_resources(crate::rtc::INDEX_PORT, 2)),
        ]
        .concat(),
    );
    let power_button = aml::device(
        "PWRB",
        [
            aml::name("_HID", aml::eisa_id("PNP0C0C")),
            aml::name("_UID", aml::integer(0)),
        ]
        .concat(),
    );
    // The Generic Event Device turns an interrupt on GED_GSI into a power-button notification.
    let ged = aml::device(
        "GED0",
        [
            aml::name("_HID", aml::string("ACPI0013")),
            aml::name("_UID", aml::integer(0)),
            aml::name("_CRS", aml::edge_interrupt_resources(GED_GSI)),
            aml::method("_EVT", 1, aml::notify("_SB_", "PWRB", NOTIFY_POWER_BUTTON)),
        ]
        .concat(),
    );
    [
        aml::name("_S5_", aml::byte_package(&[S5_SLP_TYP, 0])),
        aml::scope("_SB_", [com1, rtc, power_button, ged].concat()),
    ]
    .concat()
}

fn dsdt() -> Vec<u8> {
    let mut t = Table::new(b"DSDT", 2);
    t.bytes(&dsdt_aml());
    t.finish()
}

const FADT_HW_REDUCED_ACPI: u32 = 1 << 20;
const FADT_RESET_REG_SUP: u32 = 1 << 10;
/// IAPC_BOOT_ARCH: VGA not present; the 8042 bit stays clear (no PS/2). The CMOS RTC is present.
const IAPC_BOOT_ARCH: u16 = 1 << 2;

fn fadt(dsdt_addr: u64) -> Vec<u8> {
    let mut t = Table::new(b"FACP", 6);
    t.u32(0) // FIRMWARE_CTRL
        .u32(dsdt_addr as u32) // DSDT
        .zeros(1) // reserved
        .u8(0) // preferred PM profile: unspecified
        .u16(0) // SCI_INT: none (hardware-reduced)
        .u32(0) // SMI_CMD: ACPI is always enabled
        .zeros(4) // ACPI_ENABLE, ACPI_DISABLE, S4BIOS_REQ, PSTATE_CNT
        .zeros(4 * 8) // PM1a/b EVT, PM1a/b CNT, PM2 CNT, PM TMR, GPE0, GPE1 blocks
        .zeros(7) // PM1 EVT/CNT, PM2 CNT, PM TMR, GPE0/1 block lengths, GPE1 base
        .zeros(1) // CST_CNT
        .zeros(4 * 2) // P_LVL2/3 latency, flush size/stride
        .zeros(5) // duty offset/width, day/month alarm, century
        .u16(IAPC_BOOT_ARCH)
        .zeros(1) // reserved
        .u32(FADT_HW_REDUCED_ACPI | FADT_RESET_REG_SUP)
        .io_gas(Some(RESET_PORT))
        .u8(RESET_VALUE)
        .u16(0) // ARM boot arch
        .u8(5) // FADT minor version (6.5)
        .u64(0) // X_FIRMWARE_CTRL
        .u64(dsdt_addr) // X_DSDT
        .zeros(12 * 8) // X_PM1a/b EVT, X_PM1a/b CNT, X_PM2 CNT, X_PM TMR, X_GPE0, X_GPE1
        .io_gas(Some(SLEEP_CONTROL_PORT))
        .io_gas(Some(SLEEP_STATUS_PORT))
        .bytes(b"VIRTAX\0\0"); // hypervisor vendor identity
    t.finish()
}

fn madt(vcpus: u8) -> Vec<u8> {
    let mut t = Table::new(b"APIC", 5);
    t.u32(LOCAL_APIC_ADDR).u32(1); // flags: PCAT_COMPAT (8259 PICs present)
    for id in 0..vcpus {
        // Processor local APIC: type 0, length 8, UID, APIC ID, flags = enabled.
        t.u8(0).u8(8).u8(id).u8(id).u32(1);
    }
    // I/O APIC: type 1, length 12, ID after the CPUs, address, GSI base 0.
    t.u8(1).u8(12).u8(vcpus).u8(0).u32(IO_APIC_ADDR).u32(0);
    t.finish()
}

fn xsdt(entries: &[u64]) -> Vec<u8> {
    let mut t = Table::new(b"XSDT", 1);
    for e in entries {
        t.u64(*e);
    }
    t.finish()
}

fn rsdp(xsdt_addr: u64) -> Vec<u8> {
    let mut b = Vec::with_capacity(36);
    b.extend_from_slice(b"RSD PTR ");
    b.push(0); // checksum over the first 20 bytes
    b.extend_from_slice(OEM_ID);
    b.push(2); // revision: ACPI 2.0+
    b.extend_from_slice(&0u32.to_le_bytes()); // RSDT address: none, XSDT only
    b.extend_from_slice(&36u32.to_le_bytes());
    b.extend_from_slice(&xsdt_addr.to_le_bytes());
    b.push(0); // extended checksum over all 36 bytes
    b.extend_from_slice(&[0; 3]);
    b[8] = checksum(&b[..20]);
    b[32] = checksum(&b);
    b
}

/// Write all tables into guest memory; returns the RSDP address for `boot_params`.
pub fn write_tables(mem: &GuestMemoryMmap, vcpus: u8) -> Result<u64, String> {
    let tables = [
        (RSDP_ADDR, rsdp(XSDT_ADDR)),
        (XSDT_ADDR, xsdt(&[FADT_ADDR, MADT_ADDR])),
        (FADT_ADDR, fadt(DSDT_ADDR)),
        (DSDT_ADDR, dsdt()),
        (MADT_ADDR, madt(vcpus)),
    ];
    for w in tables.windows(2) {
        assert!(
            w[0].0 + w[0].1.len() as u64 <= w[1].0,
            "ACPI tables overlap"
        );
    }
    for (addr, bytes) in &tables {
        mem.write_slice(bytes, GuestAddress(*addr))
            .map_err(|e| format!("ACPI tables: {e}"))?;
    }
    Ok(RSDP_ADDR)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sums_to_zero(b: &[u8]) -> bool {
        b.iter().fold(0u8, |a, x| a.wrapping_add(*x)) == 0
    }

    #[test]
    fn tables_have_valid_lengths_and_checksums() {
        let fadt = fadt(DSDT_ADDR);
        assert_eq!(fadt.len(), 276, "ACPI 6.x FADT size");
        assert_eq!(
            u32::from_le_bytes(fadt[112..116].try_into().unwrap()),
            FADT_HW_REDUCED_ACPI | FADT_RESET_REG_SUP
        );
        assert_eq!(
            u64::from_le_bytes(fadt[140..148].try_into().unwrap()),
            DSDT_ADDR,
            "X_DSDT"
        );
        assert_eq!(
            &fadt[244..248],
            &[1, 8, 0, 1],
            "sleep control is a byte-wide I/O register"
        );
        assert_eq!(
            u64::from_le_bytes(fadt[248..256].try_into().unwrap()),
            u64::from(SLEEP_CONTROL_PORT)
        );

        let madt = madt(4);
        assert_eq!(madt.len(), 36 + 8 + 4 * 8 + 12);
        for t in [fadt, madt, dsdt(), xsdt(&[1, 2])] {
            assert!(sums_to_zero(&t));
            assert_eq!(
                u32::from_le_bytes(t[4..8].try_into().unwrap()) as usize,
                t.len()
            );
        }
        let r = rsdp(XSDT_ADDR);
        assert!(sums_to_zero(&r[..20]) && sums_to_zero(&r));
        assert_eq!(&r[..8], b"RSD PTR ");
    }

    #[test]
    fn dsdt_declares_s5_and_com1() {
        let aml = dsdt_aml();
        assert_eq!(
            &aml[..11],
            &[
                0x08, b'_', b'S', b'5', b'_', 0x12, 0x05, 0x02, 0x0a, 0x05, 0x00
            ]
        );
        let scope = &aml[11..];
        assert_eq!(scope[0], 0x10, "ScopeOp");
        // PkgLength: the top two bits of the lead byte count the extra length bytes.
        let extra = usize::from(scope[1] >> 6);
        let len = if extra == 0 {
            usize::from(scope[1])
        } else {
            (1..=extra).fold(usize::from(scope[1] & 0x0f), |acc, i| {
                acc | usize::from(scope[1 + i]) << (4 + 8 * (i - 1))
            })
        };
        assert_eq!(
            len,
            scope.len() - 1,
            "scope PkgLength covers the rest of the DSDT"
        );
        assert_eq!(&scope[2 + extra..6 + extra], b"_SB_");
        for device in [
            &b"COM1"[..],
            b"RTC_",
            b"PWRB",
            b"GED0",
            b"ACPI0013",
            b"_EVT",
        ] {
            assert!(
                scope.windows(device.len()).any(|w| w == device),
                "{}",
                String::from_utf8_lossy(device)
            );
        }
        assert!(
            scope
                .windows(5)
                .any(|w| w == [0x0c, 0x41, 0xd0, 0x05, 0x01]),
            "PNP0501"
        );
    }

    #[test]
    fn tables_fit_in_guest_memory() {
        let mem = crate::layout::guest_memory(64).unwrap();
        assert_eq!(write_tables(&mem, MAX_VCPUS).unwrap(), RSDP_ADDR);
        let sig: [u8; 8] = mem.read_obj(GuestAddress(RSDP_ADDR)).unwrap();
        assert_eq!(&sig, b"RSD PTR ");
    }
}
