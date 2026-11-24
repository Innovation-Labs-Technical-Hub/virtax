//! Legacy port-I/O devices: a 16550 UART on COM1 and the i8042 reset line.

use std::io::Write;

use vm_superio::{Serial, Trigger};

use crate::rtc::{self, Rtc};
use crate::x86::acpi;
use vmm_sys_util::eventfd::EventFd;

pub const COM1_BASE: u16 = 0x3f8;
pub const COM1_IRQ: u32 = 4;
const UART_PORTS: u16 = 8;
const I8042_DATA: u16 = 0x60;
const I8042_COMMAND: u16 = 0x64;
const I8042_CMD_RESET: u8 = 0xfe;

/// Raises the UART's interrupt line through a KVM irqfd.
pub struct IrqTrigger(pub EventFd);

impl Trigger for IrqTrigger {
    type E = std::io::Error;

    fn trigger(&self) -> std::io::Result<()> {
        self.0.write(1)
    }
}

/// What the run loop should do after a port write.
#[derive(Debug, PartialEq, Eq)]
pub enum PioAction {
    Continue,
    /// The guest asked to reboot (keyboard-controller reset or the ACPI reset register).
    Reset,
    /// The guest entered ACPI S5 (soft-off).
    PowerOff,
}

pub struct PioBus {
    rtc: Rtc,
    serial: Serial<IrqTrigger, vm_superio::serial::NoEvents, Box<dyn Write + Send>>,
}

impl PioBus {
    pub fn new(serial_irq: EventFd, serial_out: Box<dyn Write + Send>) -> Self {
        Self {
            rtc: Rtc::default(),
            serial: Serial::new(IrqTrigger(serial_irq), serial_out),
        }
    }

    fn uart_offset(port: u16) -> Option<u8> {
        (COM1_BASE..COM1_BASE + UART_PORTS)
            .contains(&port)
            .then(|| (port - COM1_BASE) as u8)
    }

    pub fn read(&mut self, port: u16, data: &mut [u8]) {
        match (Self::uart_offset(port), data.len()) {
            (Some(offset), 1) => data[0] = self.serial.read(offset),
            (None, 1) if port == rtc::DATA_PORT => data[0] = self.rtc.read(),
            // The i8042 status register: no data, input buffer empty.
            _ if port == I8042_COMMAND || port == I8042_DATA => data.fill(0),
            // ACPI sleep status: never asleep.
            _ if port == acpi::SLEEP_STATUS_PORT => data.fill(0),
            // Nothing decodes this port: the bus floats high.
            _ => data.fill(0xff),
        }
    }

    pub fn write(&mut self, port: u16, data: &[u8]) -> PioAction {
        match (Self::uart_offset(port), data) {
            (Some(offset), [value]) => {
                if let Err(e) = self.serial.write(offset, *value) {
                    tracing::warn!(error = ?e, "serial write failed");
                }
                PioAction::Continue
            }
            (None, [index]) if port == rtc::INDEX_PORT => {
                self.rtc.select(*index);
                PioAction::Continue
            }
            (None, [I8042_CMD_RESET]) if port == I8042_COMMAND => PioAction::Reset,
            (None, [acpi::RESET_VALUE]) if port == acpi::RESET_PORT => PioAction::Reset,
            (None, [value]) if port == acpi::SLEEP_CONTROL_PORT => {
                let slp_typ = (value >> acpi::SLP_TYP_SHIFT) & 0x7;
                if value & acpi::SLP_EN != 0 && slp_typ == acpi::S5_SLP_TYP {
                    PioAction::PowerOff
                } else {
                    PioAction::Continue
                }
            }
            _ => PioAction::Continue,
        }
    }

    pub fn flush(&mut self) {
        let _ = self.serial.writer_mut().flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    struct Sink(Arc<Mutex<Vec<u8>>>);
    impl Write for Sink {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn uart_output_reset_and_unclaimed_ports() {
        let sink = Sink::default();
        let mut bus = PioBus::new(EventFd::new(0).unwrap(), Box::new(sink.clone()));
        for b in b"hi" {
            assert_eq!(bus.write(COM1_BASE, &[*b]), PioAction::Continue);
        }
        assert_eq!(&*sink.0.lock().unwrap(), b"hi");

        let mut data = [0u8; 1];
        bus.read(0x2f8, &mut data);
        assert_eq!(data, [0xff], "no COM2");
        assert_eq!(
            bus.write(I8042_COMMAND, &[I8042_CMD_RESET]),
            PioAction::Reset
        );
        assert_eq!(bus.write(I8042_COMMAND, &[0x20]), PioAction::Continue);
        // Linux writes SLP_TYP=5 | SLP_EN to power off; other sleep states are ignored.
        assert_eq!(
            bus.write(acpi::SLEEP_CONTROL_PORT, &[0x34]),
            PioAction::PowerOff
        );
        assert_eq!(
            bus.write(acpi::SLEEP_CONTROL_PORT, &[(3 << 2) | 0x20]),
            PioAction::Continue
        );
        assert_eq!(
            bus.write(acpi::RESET_PORT, &[acpi::RESET_VALUE]),
            PioAction::Reset
        );
    }
}
