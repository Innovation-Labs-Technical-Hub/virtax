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
    run(Args::parse())
}
