//! `oxvmm`: runs one virtual machine. Guest serial output goes to `--serial` (stdout by default),
//! diagnostics go to stderr.
//!
//! Exit codes: 0 the guest powered off or rebooted, 3 the guest crashed (triple fault),
//! 1 oxvmm itself failed (bad configuration, hypervisor error).

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

/// Printed on stderr once the VM is built and the guest is about to run. `oxboxd`'s
/// `OxvmmEngine` waits for this exact line.
const READY_LINE: &str = "oxvmm: guest running";

#[derive(Parser, Debug)]
#[command(version, about = "Run one OxBox virtual machine on KVM")]
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
                    "oxvmm: cannot open serial output {}: {e}",
                    args.serial.display()
                );
                return ExitCode::from(1);
            }
        }
    };
    let config = oxvmm_core::VmConfig {
        memory_mib: args.memory_mib,
        vcpus: args.cpus,
        kernel: args.kernel,
        initrd: args.initrd,
        extra_cmdline: args.cmdline,
    };
    match oxvmm_core::run(&config, serial, || eprintln!("{READY_LINE}")) {
        Ok(oxvmm_core::VmExit::PowerOff) => {
            eprintln!("oxvmm: guest powered off; VM stopped");
            ExitCode::SUCCESS
        }
        Ok(oxvmm_core::VmExit::Reset) => {
            eprintln!("oxvmm: guest rebooted; VM stopped");
            ExitCode::SUCCESS
        }
        Ok(oxvmm_core::VmExit::TripleFault) => {
            eprintln!("oxvmm: guest crashed (triple fault)");
            ExitCode::from(3)
        }
        Err(e) => {
            eprintln!("oxvmm: {e}");
            ExitCode::from(1)
        }
    }
}

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
fn run(_args: Args) -> ExitCode {
    eprintln!("oxvmm: only Linux x86_64 hosts are supported so far");
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
