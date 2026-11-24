//! MC146818-compatible CMOS real-time clock on ports 0x70 (index) and 0x71 (data).
//!
//! The clock reads the host's UTC time on every access and reports it in BCD, 24-hour mode, so the
//! guest never sees an update in progress. BCD is not a preference: Linux on x86 always decodes the
//! CMOS clock as BCD, whatever status register B says. Alarms, periodic interrupts and CMOS RAM writes are not
//! emulated; the guest only needs the RTC to set its clock at boot.

use std::time::{SystemTime, UNIX_EPOCH};

pub const INDEX_PORT: u16 = 0x70;
pub const DATA_PORT: u16 = 0x71;

const REG_SECONDS: u8 = 0x00;
const REG_MINUTES: u8 = 0x02;
const REG_HOURS: u8 = 0x04;
const REG_WEEKDAY: u8 = 0x06;
const REG_DAY: u8 = 0x07;
const REG_MONTH: u8 = 0x08;
const REG_YEAR: u8 = 0x09;
const REG_STATUS_A: u8 = 0x0a;
const REG_STATUS_B: u8 = 0x0b;
const REG_STATUS_D: u8 = 0x0d;
const REG_CENTURY: u8 = 0x32;
/// Status B: 24-hour mode (bit 1); values in BCD (bit 2 clear).
const STATUS_B_24H_BCD: u8 = 0b0000_0010;
/// Status D: the clock's battery is good and its contents are valid.
const STATUS_D_VALID: u8 = 0x80;

/// A UTC calendar time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTime {
    pub year: u32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    /// 1 = Sunday … 7 = Saturday, as the RTC counts.
    pub weekday: u8,
}

impl DateTime {
    /// Convert seconds since the Unix epoch (proleptic Gregorian calendar, UTC).
    pub fn from_unix(secs: u64) -> Self {
        let days = (secs / 86_400) as i64;
        let rem = secs % 86_400;
        // Howard Hinnant's civil_from_days.
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097);
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
        let month = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
        let year = (yoe + era * 400 + i64::from(month <= 2)) as u32;
        Self {
            year,
            month,
            day,
            hour: (rem / 3600) as u8,
            minute: (rem % 3600 / 60) as u8,
            second: (rem % 60) as u8,
            // 1970-01-01 was a Thursday (5 in RTC numbering).
            weekday: ((days + 4).rem_euclid(7) + 1) as u8,
        }
    }

    fn now() -> Self {
        Self::from_unix(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        )
    }
}

/// Binary-coded decimal: 59 becomes 0x59.
fn bcd(v: u8) -> u8 {
    ((v / 10) << 4) | (v % 10)
}

#[derive(Default)]
pub struct Rtc {
    index: u8,
}

impl Rtc {
    /// Port 0x70 selects a register; bit 7 is the NMI mask and is ignored.
    pub fn select(&mut self, value: u8) {
        self.index = value & 0x7f;
    }

    pub fn read(&self) -> u8 {
        self.register(DateTime::now())
    }

    fn register(&self, t: DateTime) -> u8 {
        match self.index {
            REG_SECONDS => bcd(t.second),
            REG_MINUTES => bcd(t.minute),
            REG_HOURS => bcd(t.hour),
            REG_WEEKDAY => bcd(t.weekday),
            REG_DAY => bcd(t.day),
            REG_MONTH => bcd(t.month),
            REG_YEAR => bcd((t.year % 100) as u8),
            REG_CENTURY => bcd((t.year / 100) as u8),
            REG_STATUS_A => 0x26, // 32.768 kHz time base, 1024 Hz rate, no update in progress
            REG_STATUS_B => STATUS_B_24H_BCD,
            REG_STATUS_D => STATUS_D_VALID,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar_conversion_matches_known_dates() {
        let epoch = DateTime::from_unix(0);
        assert_eq!(
            (epoch.year, epoch.month, epoch.day, epoch.weekday),
            (1970, 1, 1, 5)
        );
        // 2000-02-29 12:34:56 UTC, a Tuesday.
        let leap = DateTime::from_unix(951_827_696);
        assert_eq!(
            (
                leap.year,
                leap.month,
                leap.day,
                leap.hour,
                leap.minute,
                leap.second,
                leap.weekday
            ),
            (2000, 2, 29, 12, 34, 56, 3)
        );
        // 2026-11-24 00:00:00 UTC, a Tuesday.
        let d = DateTime::from_unix(1_795_478_400);
        assert_eq!((d.year, d.month, d.day, d.weekday), (2026, 11, 24, 3));
    }

    #[test]
    fn registers_report_bcd_24h_time_and_ignore_the_nmi_bit() {
        let t = DateTime::from_unix(951_827_696);
        let mut rtc = Rtc::default();
        let mut reg = |index: u8| {
            rtc.select(index | 0x80);
            rtc.register(t)
        };
        assert_eq!(
            (reg(REG_HOURS), reg(REG_MINUTES), reg(REG_SECONDS)),
            (0x12, 0x34, 0x56)
        );
        assert_eq!(
            (
                reg(REG_YEAR),
                reg(REG_CENTURY),
                reg(REG_MONTH),
                reg(REG_DAY)
            ),
            (0x00, 0x20, 0x02, 0x29)
        );
        assert_eq!(reg(REG_STATUS_B), STATUS_B_24H_BCD);
        assert_eq!(reg(REG_STATUS_A) & 0x80, 0, "never mid-update");
    }
}
