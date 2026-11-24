//! `virtax-vmm`: runs one virtual machine. Guest serial output goes to `--serial` (stdout by default),
//! diagnostics go to stderr.
//!
//! A guest reboot restarts the VM (same process ID) unless `--no-reboot` is given. SIGTERM presses
//! the ACPI power button; SIGKILL powers off at once.
//!
//! Exit codes: 0 the guest powered off (or rebooted with `--no-reboot`), 3 the guest crashed (triple
//! fault), 1 virtax-vmm itself failed (bad configuration, hypervisor error).

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

/// Printed on stderr once the VM is built and the guest is about to run. `virtaxd`'s
/// `VirtaxVmmEngine` waits for this exact line.
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const READY_LINE: &str = "virtax-vmm: guest running";

#[derive(Parser, Debug)]
#[command(version, about = "Run one Virtax virtual machine on KVM")]
struct Args {
    /// Linux kernel: bzImage or ELF vmlinux.
    #[arg(long)]
    kernel: PathBuf,
    /// Initramfs (cpio, optionally compressed).
    #[arg(long)]
    initrd: Option<PathBuf>,
    /// Extra kernel parameters, appended to the built-in defaults.
    #[arg(long, default_value = "")]
    cmdline: String,
    #[arg(long, default_value_t = 512)]
    memory_mib: u64,
    #[arg(long, default_value_t = 1)]
    cpus: u8,
    /// Where guest serial output goes; `-` is stdout.
    #[arg(long, default_value = "-")]
    serial: PathBuf,
    /// Exit when the guest reboots instead of restarting it.
    #[arg(long)]
    no_reboot: bool,
    /// Attach a raw disk image as a virtio-blk device: `<path>` or `<path>,ro` for read-only.
    /// Repeat for more disks; the guest sees them in this order (/dev/vda, /dev/vdb, ...). Each
    /// image is locked while the VM runs: exclusively when read-write, shared when read-only.
    #[arg(long = "disk", value_name = "PATH[,ro]", value_parser = parse_disk)]
    disks: Vec<DiskArg>,
}

/// A `--disk` value, checked when the arguments are parsed.
#[derive(Debug, Clone)]
struct DiskArg {
    path: PathBuf,
    read_only: bool,
}

fn parse_disk(value: &str) -> Result<DiskArg, String> {
    let (path, read_only) = match value.rsplit_once(',') {
        Some((path, "ro")) => (path, true),
        Some((_, option)) => {
            return Err(format!(
                "unknown option `{option}` (expected `<path>` or `<path>,ro`)"
            ));
        }
        None => (value, false),
    };
    if path.is_empty() {
        return Err("missing image path".into());
    }
    Ok(DiskArg {
        path: PathBuf::from(path),
        read_only,
    })
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn run(args: Args) -> ExitCode {
    use std::io::Write;

    let serial: Box<dyn Write + Send> = if args.serial.as_os_str() == "-" {
        Box::new(std::io::stdout())
    } else {
        match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&args.serial)
        {
            Ok(f) => Box::new(f),
            Err(e) => {
                eprintln!(
                    "virtax-vmm: cannot open serial output {}: {e}",
                    args.serial.display()
                );
                return ExitCode::from(1);
            }
        }
    };
    let config = virtax_vmm_core::VmConfig {
        memory_mib: args.memory_mib,
        vcpus: args.cpus,
        kernel: args.kernel,
        initrd: args.initrd,
        extra_cmdline: args.cmdline,
        disks: args
            .disks
            .into_iter()
            .map(|d| virtax_vmm_core::disk::DiskConfig {
                path: d.path,
                read_only: d.read_only,
            })
            .collect(),
    };
    let no_reboot = args.no_reboot;
    let ready = |control: virtax_vmm_core::VmControl| {
        // SIGTERM presses the ACPI power button; SIGKILL remains the hard power-off.
        match signal_hook::iterator::Signals::new([signal_hook::consts::SIGTERM]) {
            Ok(mut signals) => {
                std::thread::spawn(move || {
                    for _ in signals.forever() {
                        eprintln!("virtax-vmm: power button pressed");
                        if let Err(e) = control.press_power_button() {
                            eprintln!("virtax-vmm: cannot press the power button: {e}");
                        }
                    }
                });
            }
            Err(e) => {
                eprintln!("virtax-vmm: SIGTERM will kill the VM instead of shutting it down: {e}")
            }
        }
        eprintln!("{READY_LINE}");
    };
    match virtax_vmm_core::run(&config, serial, ready) {
        Ok(virtax_vmm_core::VmExit::PowerOff) => {
            eprintln!("virtax-vmm: guest powered off; VM stopped");
            ExitCode::SUCCESS
        }
        Ok(virtax_vmm_core::VmExit::Reset) if no_reboot => {
            eprintln!("virtax-vmm: guest rebooted; VM stopped (--no-reboot)");
            ExitCode::SUCCESS
        }
        Ok(virtax_vmm_core::VmExit::Reset) => restart(),
        Ok(virtax_vmm_core::VmExit::TripleFault) => {
            eprintln!("virtax-vmm: guest crashed (triple fault)");
            ExitCode::from(3)
        }
        Err(e) => {
            eprintln!("virtax-vmm: {e}");
            ExitCode::from(1)
        }
    }
}

/// Start the VM again from scratch by re-executing this process with the same arguments. The PID
/// stays the same, so whoever supervises the VM sees one continuous run.
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn restart() -> ExitCode {
    use std::os::unix::process::CommandExt;

    eprintln!("virtax-vmm: guest rebooted; restarting the VM");
    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(e) => {
            eprintln!("virtax-vmm: cannot restart: {e}");
            return ExitCode::from(1);
        }
    };
    // `exec` only returns on failure.
    let e = std::process::Command::new(exe)
        .args(std::env::args_os().skip(1))
        .exec();
    eprintln!("virtax-vmm: cannot restart: {e}");
    ExitCode::from(1)
}

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
fn run(_args: Args) -> ExitCode {
    eprintln!("virtax-vmm: only Linux x86_64 hosts are supported so far");
    ExitCode::from(1)
}

fn main() -> ExitCode {
    let filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
    match Args::try_parse() {
        Ok(args) => run(args),
        Err(e) => {
            let _ = e.print();
            // --help and --version are not errors; a bad argument is a configuration error (1).
            ExitCode::from(if e.use_stderr() { 1 } else { 0 })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_disk_arguments() {
        let d = parse_disk("a.img").unwrap();
        assert_eq!((d.path, d.read_only), (PathBuf::from("a.img"), false));
        let d = parse_disk("/x/b,c.img,ro").unwrap();
        assert_eq!((d.path, d.read_only), (PathBuf::from("/x/b,c.img"), true));
        assert!(
            parse_disk("a.img,rw")
                .unwrap_err()
                .contains("unknown option")
        );
        assert!(parse_disk(",ro").is_err());
    }
}
