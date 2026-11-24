//! The few AML (ACPI Machine Language) constructs virtax-vmm's DSDT needs (ACPI 6.5 §20).

/// PkgLength: the encoded length counts itself plus `body_len` bytes.
pub fn pkg_length(body_len: usize) -> Vec<u8> {
    let one = body_len + 1;
    if one < 0x40 {
        return vec![one as u8];
    }
    let two = body_len + 2;
    if two < 0x1000 {
        return vec![0x40 | (two & 0xf) as u8, (two >> 4) as u8];
    }
    let three = body_len + 3;
    assert!(three < 0x10_0000, "AML package too large");
    vec![
        0x80 | (three & 0xf) as u8,
        (three >> 4) as u8,
        (three >> 12) as u8,
    ]
}

fn name_seg(name: &str) -> [u8; 4] {
    let b = name.as_bytes();
    assert!(
        b.len() == 4
            && b.iter()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == b'_'),
        "bad name {name}"
    );
    [b[0], b[1], b[2], b[3]]
}

fn with_pkg(op: &[u8], body: Vec<u8>) -> Vec<u8> {
    let mut out = op.to_vec();
    out.extend(pkg_length(body.len()));
    out.extend(body);
    out
}

/// `Name (NAME, value)`.
pub fn name(n: &str, value: Vec<u8>) -> Vec<u8> {
    let mut out = vec![0x08];
    out.extend(name_seg(n));
    out.extend(value);
    out
}

/// `Scope (NAME) { children }`.
pub fn scope(n: &str, children: Vec<u8>) -> Vec<u8> {
    let mut body = name_seg(n).to_vec();
    body.extend(children);
    with_pkg(&[0x10], body)
}

/// `Device (NAME) { children }`.
pub fn device(n: &str, children: Vec<u8>) -> Vec<u8> {
    let mut body = name_seg(n).to_vec();
    body.extend(children);
    with_pkg(&[0x5b, 0x82], body)
}

/// `Package { items }` of byte-sized integers.
pub fn byte_package(items: &[u8]) -> Vec<u8> {
    let mut body = vec![items.len() as u8];
    for i in items {
        body.extend(integer(u32::from(*i)));
    }
    with_pkg(&[0x12], body)
}

/// The shortest encoding of an integer constant.
pub fn integer(v: u32) -> Vec<u8> {
    match v {
        0 => vec![0x00],
        1 => vec![0x01],
        2..=0xff => vec![0x0a, v as u8],
        0x100..=0xffff => {
            let mut out = vec![0x0b];
            out.extend((v as u16).to_le_bytes());
            out
        }
        _ => {
            let mut out = vec![0x0c];
            out.extend(v.to_le_bytes());
            out
        }
    }
}

/// `EISAID ("PNP0501")`: a compressed vendor + product ID as a DWord constant.
pub fn eisa_id(id: &str) -> Vec<u8> {
    let b = id.as_bytes();
    assert!(b.len() == 7, "EISA IDs are 3 letters and 4 hex digits");
    let vendor =
        (u16::from(b[0] - 0x40) << 10) | (u16::from(b[1] - 0x40) << 5) | u16::from(b[2] - 0x40);
    let product = u16::from_str_radix(&id[3..], 16).expect("hex product ID");
    let mut out = vec![0x0c];
    out.extend(vendor.to_be_bytes());
    out.extend(product.to_be_bytes());
    out
}

/// A string constant, e.g. a `_HID` such as `"ACPI0013"`.
pub fn string(v: &str) -> Vec<u8> {
    let mut out = vec![0x0d];
    out.extend(v.as_bytes());
    out.push(0);
    out
}

fn resource_buffer(mut descriptors: Vec<u8>) -> Vec<u8> {
    descriptors.extend([0x79, 0x00]); // end tag, checksum not used
    let mut body = integer(descriptors.len() as u32);
    body.extend(descriptors);
    with_pkg(&[0x11], body)
}

fn io_descriptor(port: u16, len: u8) -> Vec<u8> {
    let mut d = vec![0x47, 0x01];
    d.extend(port.to_le_bytes());
    d.extend(port.to_le_bytes());
    d.extend([0x01, len]);
    d
}

/// `ResourceTemplate () { IO (Decode16, port, port, 1, len) }`.
pub fn io_resources(port: u16, len: u8) -> Vec<u8> {
    resource_buffer(io_descriptor(port, len))
}

/// `ResourceTemplate () { IO (Decode16, port, port, 1, len) IRQNoFlags () { irq } }`.
pub fn io_irq_resources(port: u16, len: u8, irq: u8) -> Vec<u8> {
    assert!(irq < 16, "IRQNoFlags covers ISA IRQs 0-15");
    let mut d = io_descriptor(port, len);
    d.push(0x22); // IRQ descriptor without flags: edge-triggered, active-high
    d.extend((1u16 << irq).to_le_bytes());
    resource_buffer(d)
}

/// `ResourceTemplate () { Interrupt (ResourceConsumer, Edge, ActiveHigh, Exclusive) { gsi } }`.
pub fn edge_interrupt_resources(gsi: u32) -> Vec<u8> {
    // Extended interrupt descriptor: flags bit 0 = consumer, bit 1 = edge; one interrupt.
    let mut d = vec![0x89];
    d.extend(6u16.to_le_bytes());
    d.extend([0x03, 0x01]);
    d.extend(gsi.to_le_bytes());
    resource_buffer(d)
}

/// `ResourceTemplate () { descriptors }` from descriptors built by the `*_descriptor` helpers.
pub fn resource_template(descriptors: &[Vec<u8>]) -> Vec<u8> {
    resource_buffer(descriptors.concat())
}

/// `WordBusNumber (ResourceProducer, MinFixed, MaxFixed, PosDecode, 0, first, last, 0, count)`.
pub fn bus_number_descriptor(first: u16, last: u16) -> Vec<u8> {
    // Word address space descriptor: type 2 (bus numbers), producer with fixed min and max.
    let mut d = vec![0x88];
    d.extend(13u16.to_le_bytes());
    d.extend([0x02, 0x0c, 0x00]);
    for v in [0, first, last, 0, last - first + 1] {
        d.extend(v.to_le_bytes());
    }
    d
}

/// `DWordMemory (ResourceProducer, PosDecode, MinFixed, MaxFixed, NonCacheable, ReadWrite, 0, base,
/// base + len - 1, 0, len)`: a memory window a bridge forwards.
pub fn memory_window_descriptor(base: u32, len: u32) -> Vec<u8> {
    // DWord address space descriptor: type 0 (memory), producer with fixed min and max, read-write.
    let mut d = vec![0x87];
    d.extend(23u16.to_le_bytes());
    d.extend([0x00, 0x0c, 0x01]);
    for v in [0, base, base + (len - 1), 0, len] {
        d.extend(v.to_le_bytes());
    }
    d
}

/// `Memory32Fixed (ReadWrite, base, len)`.
pub fn memory32_fixed_descriptor(base: u32, len: u32) -> Vec<u8> {
    let mut d = vec![0x86];
    d.extend(9u16.to_le_bytes());
    d.push(0x01);
    d.extend(base.to_le_bytes());
    d.extend(len.to_le_bytes());
    d
}

/// `Method (NAME, args) { body }`, not serialized.
pub fn method(n: &str, args: u8, body: Vec<u8>) -> Vec<u8> {
    assert!(args <= 7, "AML methods take at most 7 arguments");
    let mut b = name_seg(n).to_vec();
    b.push(args);
    b.extend(body);
    with_pkg(&[0x14], b)
}

/// `Notify (\SCOPE.NAME, value)`.
pub fn notify(scope: &str, device: &str, value: u8) -> Vec<u8> {
    let mut out = vec![0x86, b'\\', 0x2e];
    out.extend(name_seg(scope));
    out.extend(name_seg(device));
    out.extend(integer(u32::from(value)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkg_length_boundaries() {
        assert_eq!(pkg_length(4), vec![5]);
        assert_eq!(pkg_length(62), vec![63]);
        assert_eq!(pkg_length(62 + 1), vec![0x40 | (65 & 0xf), (65 >> 4) as u8]);
    }

    #[test]
    fn eisa_id_matches_the_well_known_value() {
        // PNP0501 is 0x0105D041 as a little-endian DWord.
        assert_eq!(eisa_id("PNP0501"), vec![0x0c, 0x41, 0xd0, 0x05, 0x01]);
    }

    #[test]
    fn s5_package_matches_hand_assembled_aml() {
        let aml = name("_S5_", byte_package(&[5, 0]));
        assert_eq!(
            aml,
            vec![
                0x08, b'_', b'S', b'5', b'_', 0x12, 0x05, 0x02, 0x0a, 0x05, 0x00
            ]
        );
    }

    #[test]
    fn power_button_pieces() {
        assert_eq!(
            string("ACPI0013"),
            [&[0x0d][..], b"ACPI0013", &[0]].concat()
        );
        let irq = edge_interrupt_resources(5);
        assert_eq!(&irq[4..13], &[0x89, 6, 0, 0x03, 1, 5, 0, 0, 0]);
        assert_eq!(
            notify("_SB_", "PWRB", 0x80),
            [&[0x86, b'\\', 0x2e][..], b"_SB_PWRB", &[0x0a, 0x80]].concat()
        );
        let m = method("_EVT", 1, notify("_SB_", "PWRB", 0x80));
        assert_eq!(
            (m[0], m[1] as usize, &m[2..6], m[6]),
            (0x14, m.len() - 1, &b"_EVT"[..], 1)
        );
        assert_eq!(
            &io_resources(0x70, 2)[4..12],
            &[0x47, 0x01, 0x70, 0x00, 0x70, 0x00, 0x01, 0x02]
        );
    }

    #[test]
    fn pci_host_bridge_resources() {
        // Byte layouts from ACPI 6.5 §6.4.3.5.2 (DWord), §6.4.3.5.3 (Word) and §6.4.3.4 (Memory32Fixed).
        assert_eq!(
            bus_number_descriptor(0, 0),
            [0x88, 13, 0, 2, 0x0c, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0]
        );
        let mem = memory_window_descriptor(0xc000_0000, 0x1000_0000);
        assert_eq!(&mem[..6], &[0x87, 23, 0, 0, 0x0c, 1]);
        assert_eq!(&mem[10..14], &0xc000_0000u32.to_le_bytes(), "minimum");
        assert_eq!(&mem[14..18], &0xcfff_ffffu32.to_le_bytes(), "maximum");
        assert_eq!(&mem[22..26], &0x1000_0000u32.to_le_bytes(), "length");
        assert_eq!(mem.len(), 26);
        assert_eq!(
            memory32_fixed_descriptor(0xfeb0_0000, 0x10_0000),
            [0x86, 9, 0, 1, 0, 0, 0xb0, 0xfe, 0, 0, 0x10, 0]
        );
        let t = resource_template(&[bus_number_descriptor(0, 0), mem]);
        // BufferOp, PkgLength, BufferSize = 16 + 26 + 2 (end tag).
        assert_eq!(&t[..4], &[0x11, 0x2f, 0x0a, 44]);
        assert_eq!(&t[t.len() - 2..], &[0x79, 0]);
    }

    #[test]
    fn com1_resources() {
        let buf = io_irq_resources(0x3f8, 8, 4);
        // BufferOp, PkgLength, BufferSize (13), IO descriptor, IRQ descriptor, end tag.
        assert_eq!(&buf[..4], &[0x11, 0x10, 0x0a, 13]);
        assert_eq!(
            &buf[4..12],
            &[0x47, 0x01, 0xf8, 0x03, 0xf8, 0x03, 0x01, 0x08]
        );
        assert_eq!(&buf[12..], &[0x22, 0x10, 0x00, 0x79, 0x00]);
    }
}
