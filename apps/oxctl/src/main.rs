//! `oxctl`: OxBox command-line client.
//!
//! Exit codes: 0 ok, 1 generic, 2 usage, 3 not found, 4 conflict/etag, 5 state/precondition,
//! 6 timeout, 7 cancelled, 10 engine error. With `--json`, stdout carries only JSON (errors too).

// A CLI builds at most one error per process; boxing it would only add noise.
#![allow(clippy::result_large_err)]

mod output;

use std::io::{IsTerminal, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use clap::{Args, Parser, Subcommand};
use oxbox_api::v1 as pb;
use oxbox_api::v1::daemon_client::DaemonClient;
use oxbox_api::v1::events_client::EventsClient;
use oxbox_api::v1::machines_client::MachinesClient;
use oxbox_api::v1::operations_client::OperationsClient;
use oxbox_api::v1::runtime_client::RuntimeClient;
use oxbox_model::OxeCode;
use serde_json::json;
use tonic::transport::Channel;
use tonic::{Code, Status};

#[derive(Parser)]
#[command(name = "oxctl", version, about = "Control OxBox virtual machines")]
struct Cli {
    /// Daemon socket (default: $XDG_RUNTIME_DIR/oxbox/oxboxd.sock).
    #[arg(long, global = true, env = "OXBOX_SOCKET")]
    socket: Option<PathBuf>,
    /// Print machine-readable JSON on stdout.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Machines.
    #[command(subcommand)]
    Vm(VmCmd),
    /// Long-running operations.
    #[command(subcommand)]
    Ops(OpsCmd),
    /// Change events.
    #[command(subcommand)]
    Events(EventsCmd),
    /// Client, daemon, API and schema versions.
    Version,
}

#[derive(Subcommand)]
enum VmCmd {
    /// List machines.
    List {
        #[arg(long)]
        state: Option<String>,
        #[arg(long)]
        group: Option<String>,
        /// Print names only.
        #[arg(short, long)]
        quiet: bool,
    },
    /// Show one machine.
    Show { vm: String },
    /// Create a machine.
    Create {
        #[arg(long)]
        name: String,
        #[arg(long)]
        engine: Option<String>,
        #[arg(long)]
        folder: Option<PathBuf>,
        #[command(flatten)]
        config: Box<ConfigArgs>,
    },
    /// Change machine settings.
    Set {
        vm: String,
        #[arg(long)]
        name: Option<String>,
        #[command(flatten)]
        config: Box<ConfigArgs>,
        /// Fail if the machine changed since this etag (default: the current etag).
        #[arg(long)]
        etag: Option<String>,
        /// Report what would change and when it applies, without changing anything.
        #[arg(long)]
        dry_run: bool,
        /// On a running machine, queue changes that need power-off instead of refusing them.
        /// Changes that take effect at the next boot are always queued.
        #[arg(long)]
        at_shutdown: bool,
    },
    /// Changes queued until the machine powers off.
    #[command(subcommand)]
    Pending(PendingCmd),
    /// Start a machine.
    Start {
        vm: String,
        #[command(flatten)]
        wait: WaitArgs,
    },
    /// Shut down a machine (ACPI), or power it off with --force.
    Stop {
        vm: String,
        #[arg(long)]
        force: bool,
        #[command(flatten)]
        wait: WaitArgs,
    },
    /// Unregister a machine; --delete-files also removes its folder (needs --yes).
    Delete {
        vm: String,
        #[arg(long)]
        delete_files: bool,
        #[arg(long)]
        yes: bool,
    },
    /// Validate a machine.toml locally, reporting line and column for each problem.
    Validate { path: PathBuf },
    /// Print the JSON Schema of machine.toml.
    Schema,
}

#[derive(Args, Default)]
struct ConfigArgs {
    #[arg(long)]
    os_type: Option<String>,
    #[arg(long)]
    cpus: Option<u32>,
    /// Memory in MiB.
    #[arg(long)]
    memory: Option<u64>,
    /// Video memory in MiB.
    #[arg(long)]
    vram: Option<u32>,
    #[arg(long)]
    monitors: Option<u32>,
    /// bios or efi.
    #[arg(long)]
    firmware: Option<String>,
    #[arg(long)]
    secure_boot: Option<bool>,
    /// Group path such as /lab (repeatable; replaces the list).
    #[arg(long = "group")]
    groups: Vec<String>,
    /// Tag (repeatable; replaces the list).
    #[arg(long = "tag")]
    tags: Vec<String>,
    /// Kernel for direct boot (oxvmm): bzImage or vmlinux. An empty value removes direct boot.
    #[arg(long)]
    kernel: Option<String>,
    /// Initramfs for direct boot.
    #[arg(long)]
    initrd: Option<String>,
    /// Extra kernel parameters for direct boot.
    #[arg(long)]
    cmdline: Option<String>,
}

#[derive(Args)]
struct WaitArgs {
    /// Return the operation name immediately instead of waiting.
    #[arg(long)]
    no_wait: bool,
    /// Fail (exit 5) if the machine is already in the requested state.
    #[arg(long)]
    strict: bool,
}

#[derive(Subcommand)]
enum PendingCmd {
    /// List queued changes.
    List { vm: String },
    /// Drop one queued change, e.g. `memory_mib`.
    Cancel { vm: String, field: String },
}

#[derive(Subcommand)]
enum EventsCmd {
    /// Stream events (NDJSON with --json). Reconnects and resumes by seq if the daemon restarts.
    Watch {
        /// Replay events after this seq first (0 = everything retained). Default: from now.
        #[arg(long)]
        since: Option<u64>,
        /// Resource prefix such as machines/ (repeatable).
        #[arg(long = "resource")]
        resources: Vec<String>,
        /// Event kind; `machine` matches every machine.* kind (repeatable).
        #[arg(long = "kind")]
        kinds: Vec<String>,
        /// Exit after printing this many events.
        #[arg(long)]
        count: Option<u64>,
    },
}

#[derive(Subcommand)]
enum OpsCmd {
    List {
        #[arg(long, default_value_t = 20)]
        limit: u32,
    },
    Get {
        name: String,
    },
    Wait {
        name: String,
    },
    Cancel {
        name: String,
    },
}

struct CliError {
    exit: u8,
    code: Option<OxeCode>,
    raw_code: Option<String>,
    message: String,
    etag: Option<String>,
    /// The failure was already written to stdout as JSON; only the exit code remains.
    reported: bool,
}

impl CliError {
    fn new(exit: u8, message: impl Into<String>) -> Self {
        Self {
            exit,
            code: None,
            raw_code: None,
            message: message.into(),
            etag: None,
            reported: false,
        }
    }

    fn usage(message: impl Into<String>) -> Self {
        Self::new(2, message)
    }

    fn from_oxe_code(raw: &str, message: String) -> Self {
        let code = OxeCode::by_code(raw);
        let exit = match code {
            Some(OxeCode::RT_CANCELLED) => 7,
            Some(
                OxeCode::RT_INVALID_TRANSITION
                | OxeCode::RT_NEEDS_POWER_OFF
                | OxeCode::RT_ALREADY_IN_STATE,
            ) => 5,
            _ => 10,
        };
        Self {
            exit,
            code,
            raw_code: Some(raw.to_string()),
            message,
            etag: None,
            reported: false,
        }
    }
}

impl From<Status> for CliError {
    fn from(s: Status) -> Self {
        let meta = |k: &str| {
            s.metadata()
                .get(k)
                .and_then(|v| v.to_str().ok())
                .map(str::to_string)
        };
        let raw_code = meta(oxbox_api::OXE_CODE_KEY);
        let code = raw_code.as_deref().and_then(OxeCode::by_code);
        let exit = match s.code() {
            Code::InvalidArgument => 2,
            Code::NotFound => 3,
            Code::Aborted | Code::AlreadyExists => 4,
            Code::FailedPrecondition
                if raw_code
                    .as_deref()
                    .is_some_and(|c| c.starts_with("OXE-ENG")) =>
            {
                10
            }
            Code::FailedPrecondition | Code::OutOfRange => 5,
            Code::DeadlineExceeded => 6,
            Code::Cancelled => 7,
            Code::Internal
                if raw_code
                    .as_deref()
                    .is_some_and(|c| c.starts_with("OXE-ENG")) =>
            {
                10
            }
            _ => 1,
        };
        let message = if s.code() == Code::Unavailable {
            format!("cannot reach oxboxd: {}", s.message())
        } else {
            s.message().to_string()
        };
        Self {
            exit,
            code,
            raw_code,
            message,
            etag: meta(oxbox_api::OXE_ETAG_KEY),
            reported: false,
        }
    }
}

type Result<T> = std::result::Result<T, CliError>;

struct Ctx {
    json: bool,
    channel: Channel,
    socket: PathBuf,
}

impl Ctx {
    fn machines(&self) -> MachinesClient<Channel> {
        MachinesClient::new(self.channel.clone())
    }
    fn runtime(&self) -> RuntimeClient<Channel> {
        RuntimeClient::new(self.channel.clone())
    }
    fn ops(&self) -> OperationsClient<Channel> {
        OperationsClient::new(self.channel.clone())
    }

    fn print(&self, value: serde_json::Value, human: impl FnOnce() -> String) {
        if self.json {
            println!("{value}");
        } else {
            println!("{}", human());
        }
    }
}

fn config_and_mask(
    name: Option<String>,
    c: ConfigArgs,
) -> Result<(pb::MachineConfig, Vec<String>)> {
    let mut cfg = pb::MachineConfig::default();
    let mut mask = Vec::new();
    let mut set = |path: &str| mask.push(path.to_string());
    if let Some(v) = name {
        cfg.display_name = v;
        set("display_name");
    }
    if let Some(v) = c.os_type {
        cfg.os_type = v;
        set("os_type");
    }
    if let Some(v) = c.cpus {
        cfg.cpu_count = v;
        set("cpu_count");
    }
    if let Some(v) = c.memory {
        cfg.memory_mib = v;
        set("memory_mib");
    }
    if let Some(v) = c.vram {
        cfg.vram_mib = v;
        set("vram_mib");
    }
    if let Some(v) = c.monitors {
        cfg.monitors = v;
        set("monitors");
    }
    if let Some(v) = c.firmware {
        cfg.firmware = v;
        set("firmware");
    }
    if let Some(v) = c.secure_boot {
        cfg.secure_boot = v;
        set("secure_boot");
    }
    if !c.groups.is_empty() {
        cfg.groups = c.groups;
        set("groups");
    }
    if !c.tags.is_empty() {
        cfg.tags = c.tags;
        set("tags");
    }
    // Paths are sent as given; make them absolute so they do not depend on the daemon's directory.
    let absolute = |p: String| -> String {
        let path = PathBuf::from(&p);
        if p.is_empty() || path.is_absolute() {
            p
        } else {
            std::env::current_dir()
                .map(|d| d.join(path).to_string_lossy().into_owned())
                .unwrap_or(p)
        }
    };
    if let Some(v) = c.kernel {
        cfg.boot_kernel = absolute(v);
        set("boot_kernel");
    }
    if let Some(v) = c.initrd {
        cfg.boot_initrd = absolute(v);
        set("boot_initrd");
    }
    if let Some(v) = c.cmdline {
        cfg.boot_cmdline = v;
        set("boot_cmdline");
    }
    Ok((cfg, mask))
}

/// Follow an operation to completion, showing progress on a terminal.
async fn wait_op(ctx: &Ctx, name: &str) -> Result<pb::Operation> {
    let mut stream = ctx
        .ops()
        .watch_operation(pb::WatchOperationRequest { name: name.into() })
        .await?
        .into_inner();
    let tty = !ctx.json && std::io::stderr().is_terminal();
    let mut last = None;
    while let Some(op) = stream.message().await? {
        if tty {
            eprint!("\r\x1b[2K{} {:>3}% {}", op.kind, op.percent, op.step);
            let _ = std::io::stderr().flush();
        }
        let done = op.done;
        last = Some(op);
        if done {
            break;
        }
    }
    if tty {
        eprintln!();
    }
    let op = last.ok_or_else(|| CliError::new(1, "operation stream ended early"))?;
    if !op.done {
        return Err(CliError::new(1, format!("{name} did not finish")));
    }
    Ok(op)
}

/// Report an operation: done → print result (error → exit code), not waiting → print its name.
fn finish_op(ctx: &Ctx, op: &pb::Operation) -> Result<()> {
    if let Some(e) = &op.error
        && op.state != "succeeded"
    {
        let message = if e.detail.is_empty() {
            format!("{}: {}", e.code, e.message)
        } else {
            format!("{}: {} ({})", e.code, e.message, e.detail)
        };
        if ctx.json {
            println!("{}", output::operation(op));
        }
        let mut err = CliError::from_oxe_code(&e.code, message);
        err.exit = if op.state == "cancelled" { 7 } else { err.exit };
        err.reported = ctx.json;
        return Err(err);
    }
    ctx.print(output::operation(op), || {
        if op.done {
            format!("{} {}: {}", op.kind, op.target, op.state)
        } else {
            op.name.clone()
        }
    });
    Ok(())
}

async fn action(ctx: &Ctx, vm: String, start: bool, force: bool, wait: WaitArgs) -> Result<()> {
    let res = if start {
        ctx.runtime()
            .start(pb::StartRequest {
                name: vm.clone(),
                headless: true,
            })
            .await
    } else {
        ctx.runtime()
            .stop(pb::StopRequest {
                name: vm.clone(),
                force,
            })
            .await
    };
    let op = match res {
        Ok(r) => r.into_inner(),
        Err(s) => {
            let err = CliError::from(s);
            if err.code == Some(OxeCode::RT_ALREADY_IN_STATE) && !wait.strict {
                let state = if start { "running" } else { "powered_off" };
                ctx.print(
                    json!({"name": vm, "state": state, "changed": false}),
                    || format!("{vm} is already {state}"),
                );
                return Ok(());
            }
            return Err(err);
        }
    };
    if wait.no_wait {
        ctx.print(output::operation(&op), || op.name.clone());
        return Ok(());
    }
    let op = wait_op(ctx, &op.name).await?;
    finish_op(ctx, &op)
}

async fn run_vm(ctx: &Ctx, cmd: VmCmd) -> Result<()> {
    match cmd {
        VmCmd::List {
            state,
            group,
            quiet,
        } => {
            let r = ctx
                .machines()
                .list_machines(pb::ListMachinesRequest {
                    state: state.unwrap_or_default(),
                    group: group.unwrap_or_default(),
                })
                .await?
                .into_inner();
            if ctx.json {
                println!(
                    "{}",
                    json!(r.machines.iter().map(output::summary).collect::<Vec<_>>())
                );
            } else if quiet {
                for m in &r.machines {
                    println!("{}", m.display_name);
                }
            } else {
                let rows: Vec<Vec<String>> = r
                    .machines
                    .iter()
                    .map(|m| {
                        vec![
                            m.display_name.clone(),
                            if m.accessible && m.pending_change_count > 0 {
                                format!("{} ({} pending)", m.state, m.pending_change_count)
                            } else if m.accessible {
                                m.state.clone()
                            } else {
                                format!("inaccessible ({})", m.error_code)
                            },
                            m.os_type.clone(),
                            m.cpu_count.to_string(),
                            format!("{} MiB", m.memory_mib),
                            m.engine.clone(),
                            m.id.clone(),
                        ]
                    })
                    .collect();
                println!(
                    "{}",
                    output::table(
                        &["NAME", "STATE", "OS", "CPUS", "MEMORY", "ENGINE", "ID"],
                        &rows
                    )
                );
            }
        }
        VmCmd::Show { vm } => {
            let m = ctx
                .machines()
                .get_machine(pb::GetMachineRequest { name: vm })
                .await?
                .into_inner();
            ctx.print(output::machine(&m), || {
                let state = m
                    .summary
                    .as_ref()
                    .map(|s| s.state.clone())
                    .unwrap_or_default();
                let mut out = format!(
                    "# {} ({state}, etag {})\n# folder: {}\n",
                    m.name, m.etag, m.folder
                );
                for p in &m.pending_changes {
                    out.push_str(&format!(
                        "# pending: {} = {} ({}, queued {})\n",
                        p.field_path,
                        p.value_json,
                        output::apply_mode(p.apply_mode),
                        p.created_at
                    ));
                }
                out.push_str(m.manifest_toml.trim_end());
                out
            });
        }
        VmCmd::Create {
            name,
            engine,
            folder,
            config,
        } => {
            let (mut cfg, _) = config_and_mask(Some(name), *config)?;
            cfg.display_name = cfg.display_name.trim().to_string();
            let m = ctx
                .machines()
                .create_machine(pb::CreateMachineRequest {
                    config: Some(cfg),
                    engine: engine.unwrap_or_default(),
                    folder: folder
                        .map(|f| f.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                })
                .await?
                .into_inner();
            ctx.print(output::machine(&m), || {
                format!(
                    "created {} ({})",
                    m.config
                        .as_ref()
                        .map(|c| c.display_name.as_str())
                        .unwrap_or(""),
                    m.name
                )
            });
        }
        VmCmd::Set {
            vm,
            name,
            config,
            etag,
            dry_run,
            at_shutdown,
        } => {
            let (cfg, mask) = config_and_mask(name, *config)?;
            if mask.is_empty() {
                return Err(CliError::usage(
                    "nothing to change; pass at least one setting such as --cpus",
                ));
            }
            let etag = match etag {
                Some(e) => e,
                None => {
                    ctx.machines()
                        .get_machine(pb::GetMachineRequest { name: vm.clone() })
                        .await?
                        .into_inner()
                        .etag
                }
            };
            let r = ctx
                .machines()
                .update_machine(pb::UpdateMachineRequest {
                    name: vm,
                    config: Some(cfg),
                    update_mask: Some(prost_types::FieldMask { paths: mask }),
                    etag,
                    validate_only: dry_run,
                    apply_policy: if at_shutdown {
                        pb::ApplyPolicy::Queue
                    } else {
                        pb::ApplyPolicy::Unspecified
                    } as i32,
                })
                .await?
                .into_inner();
            let results: Vec<_> = r.field_results.iter().map(output::field_result).collect();
            let machine = r.machine.as_ref().map(output::machine);
            ctx.print(
                json!({"dry_run": dry_run, "machine": machine, "field_results": results}),
                || {
                    let rows: Vec<Vec<String>> = r
                        .field_results
                        .iter()
                        .map(|f| {
                            vec![
                                f.path.clone(),
                                if f.queued {
                                    format!("{} (queued)", output::apply_mode(f.apply_mode))
                                } else {
                                    output::apply_mode(f.apply_mode).into()
                                },
                                f.issues.join("; "),
                            ]
                        })
                        .collect();
                    let head = if dry_run {
                        "dry run, nothing changed"
                    } else {
                        "updated"
                    };
                    format!(
                        "{head} (etag {})\n{}",
                        r.machine.as_ref().map(|m| m.etag.as_str()).unwrap_or(""),
                        output::table(&["FIELD", "APPLIES", "ISSUES"], &rows)
                    )
                },
            );
        }
        VmCmd::Start { vm, wait } => action(ctx, vm, true, false, wait).await?,
        VmCmd::Stop { vm, force, wait } => action(ctx, vm, false, force, wait).await?,
        VmCmd::Delete {
            vm,
            delete_files,
            yes,
        } => {
            if delete_files && !yes {
                return Err(CliError::usage(
                    "--delete-files removes the machine folder and its disks; add --yes to confirm",
                ));
            }
            ctx.machines()
                .delete_machine(pb::DeleteMachineRequest {
                    name: vm.clone(),
                    delete_files,
                })
                .await?;
            ctx.print(
                json!({"name": vm, "deleted": true, "files_deleted": delete_files}),
                || format!("deleted {vm}"),
            );
        }
        VmCmd::Pending(PendingCmd::List { vm }) => {
            let m = ctx
                .machines()
                .get_machine(pb::GetMachineRequest { name: vm })
                .await?
                .into_inner();
            print_pending(ctx, &m);
        }
        VmCmd::Pending(PendingCmd::Cancel { vm, field }) => {
            let m = ctx
                .machines()
                .cancel_pending_change(pb::CancelPendingChangeRequest {
                    name: vm,
                    field_path: field,
                })
                .await?
                .into_inner();
            print_pending(ctx, &m);
        }
        VmCmd::Validate { .. } | VmCmd::Schema => {
            unreachable!("handled without a daemon connection")
        }
    }
    Ok(())
}

fn print_pending(ctx: &Ctx, m: &pb::Machine) {
    ctx.print(
        json!(
            m.pending_changes
                .iter()
                .map(output::pending)
                .collect::<Vec<_>>()
        ),
        || {
            if m.pending_changes.is_empty() {
                return "no pending changes".into();
            }
            let rows: Vec<Vec<String>> = m
                .pending_changes
                .iter()
                .map(|p| {
                    vec![
                        p.field_path.clone(),
                        p.value_json.clone(),
                        output::apply_mode(p.apply_mode).into(),
                        p.created_at.clone(),
                    ]
                })
                .collect();
            output::table(&["FIELD", "VALUE", "APPLIES", "QUEUED AT"], &rows)
        },
    );
}

async fn run_ops(ctx: &Ctx, cmd: OpsCmd) -> Result<()> {
    match cmd {
        OpsCmd::List { limit } => {
            let r = ctx
                .ops()
                .list_operations(pb::ListOperationsRequest { page_size: limit })
                .await?
                .into_inner();
            ctx.print(
                json!(
                    r.operations
                        .iter()
                        .map(output::operation)
                        .collect::<Vec<_>>()
                ),
                || {
                    let rows: Vec<Vec<String>> = r
                        .operations
                        .iter()
                        .map(|o| {
                            vec![
                                o.name.clone(),
                                o.kind.clone(),
                                o.target.clone(),
                                o.state.clone(),
                                format!("{}%", o.percent),
                            ]
                        })
                        .collect();
                    output::table(&["NAME", "KIND", "TARGET", "STATE", "PROGRESS"], &rows)
                },
            );
        }
        OpsCmd::Get { name } => {
            let op = ctx
                .ops()
                .get_operation(pb::GetOperationRequest { name })
                .await?
                .into_inner();
            ctx.print(output::operation(&op), || {
                format!(
                    "{} {} {} {}% {}",
                    op.name, op.kind, op.state, op.percent, op.step
                )
            });
        }
        OpsCmd::Wait { name } => {
            let op = wait_op(ctx, &name).await?;
            finish_op(ctx, &op)?;
        }
        OpsCmd::Cancel { name } => {
            let op = ctx
                .ops()
                .cancel_operation(pb::CancelOperationRequest { name })
                .await?
                .into_inner();
            ctx.print(output::operation(&op), || {
                format!("{} {}", op.name, op.state)
            });
        }
    }
    Ok(())
}

const RECONNECT_MIN: Duration = Duration::from_millis(100);
const RECONNECT_MAX: Duration = Duration::from_millis(500);

fn print_event(ctx: &Ctx, e: &pb::Event) {
    if ctx.json {
        let payload: serde_json::Value =
            serde_json::from_str(&e.payload_json).unwrap_or(serde_json::Value::Null);
        println!(
            "{}",
            json!({"seq": e.seq, "ts": e.ts, "resource": e.resource, "kind": e.kind, "payload": payload})
        );
    } else {
        println!(
            "{:>6}  {}  {:<22} {}  {}",
            e.seq, e.ts, e.kind, e.resource, e.payload_json
        );
    }
    let _ = std::io::stdout().flush();
}

/// Follow `Events.Watch`, resuming from the last printed seq after a disconnect, so a daemon
/// restart neither loses nor repeats events.
async fn watch_events(
    ctx: &Ctx,
    since: Option<u64>,
    resources: Vec<String>,
    kinds: Vec<String>,
    count: Option<u64>,
) -> Result<()> {
    // Pin "now" to a seq so even the first reconnect resumes exactly.
    let mut cursor = match since {
        Some(s) => s,
        None => {
            DaemonClient::new(ctx.channel.clone())
                .get_version(pb::GetVersionRequest {})
                .await?
                .into_inner()
                .last_event_seq
        }
    };
    let mut printed = 0u64;
    let mut channel = ctx.channel.clone();
    loop {
        let req = pb::WatchEventsRequest {
            since_seq: Some(cursor),
            resource_prefixes: resources.clone(),
            kinds: kinds.clone(),
        };
        match EventsClient::new(channel.clone()).watch(req).await {
            Err(s) if s.code() == Code::Unavailable || s.code() == Code::Unknown => {}
            Err(s) => return Err(s.into()),
            Ok(resp) => {
                let mut stream = resp.into_inner();
                loop {
                    match stream.message().await {
                        Ok(Some(e)) => {
                            cursor = e.seq;
                            print_event(ctx, &e);
                            printed += 1;
                            if count.is_some_and(|c| printed >= c) {
                                return Ok(());
                            }
                        }
                        Ok(None) => break,
                        Err(s) if s.code() == Code::OutOfRange => return Err(s.into()),
                        Err(_) => break,
                    }
                }
            }
        };
        if !ctx.json {
            eprintln!("disconnected from oxboxd; reconnecting from seq {cursor}");
        }
        let mut delay = RECONNECT_MIN;
        channel = loop {
            tokio::time::sleep(delay).await;
            if let Ok(c) = oxbox_api::connect(&ctx.socket).await {
                break c;
            }
            delay = (delay * 2).min(RECONNECT_MAX);
        };
    }
}

fn validate_local(path: &PathBuf, json_out: bool) -> Result<()> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| CliError::new(3, format!("{}: {e}", path.display())))?;
    match oxbox_model::parse_manifest(&text) {
        Ok(_) => {
            if json_out {
                println!("{}", json!({"valid": true, "issues": []}));
            } else {
                println!("{}: valid", path.display());
            }
            Ok(())
        }
        Err(issues) => {
            if json_out {
                println!("{}", json!({"valid": false, "issues": issues}));
            } else {
                for i in &issues {
                    eprintln!("{}:{i}", path.display());
                }
            }
            Err(CliError {
                code: Some(OxeCode::CFG_INVALID_MANIFEST),
                reported: json_out,
                ..CliError::new(1, format!("{} problem(s) found", issues.len()))
            })
        }
    }
}

async fn run(cli: Cli) -> Result<()> {
    match &cli.cmd {
        Cmd::Vm(VmCmd::Validate { path }) => return validate_local(path, cli.json),
        Cmd::Vm(VmCmd::Schema) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&oxbox_model::json_schema())
                    .expect("schema serializes")
            );
            return Ok(());
        }
        _ => {}
    }
    let socket = cli.socket.unwrap_or_else(oxbox_api::default_socket_path);
    let channel = oxbox_api::connect(&socket).await.map_err(|e| {
        CliError::new(
            1,
            format!(
                "cannot reach oxboxd at {}: {e}; is the daemon running?",
                socket.display()
            ),
        )
    })?;
    let ctx = Ctx {
        json: cli.json,
        channel,
        socket: socket.clone(),
    };
    match cli.cmd {
        Cmd::Vm(cmd) => run_vm(&ctx, cmd).await,
        Cmd::Ops(cmd) => run_ops(&ctx, cmd).await,
        Cmd::Events(EventsCmd::Watch {
            since,
            resources,
            kinds,
            count,
        }) => watch_events(&ctx, since, resources, kinds, count).await,
        Cmd::Version => {
            let v = DaemonClient::new(ctx.channel.clone())
                .get_version(pb::GetVersionRequest {})
                .await?
                .into_inner();
            ctx.print(
                json!({
                    "oxctl": env!("CARGO_PKG_VERSION"),
                    "daemon": v.daemon_version,
                    "api": v.api_version,
                    "manifest_schema": v.manifest_schema,
                    "registry_schema": v.registry_schema,
                    "engines": v.engines,
                }),
                || {
                    format!(
                        "oxctl {}\noxboxd {} ({}, {}, registry v{})\nengines: {}",
                        env!("CARGO_PKG_VERSION"),
                        v.daemon_version,
                        v.api_version,
                        v.manifest_schema,
                        v.registry_schema,
                        if v.engines.is_empty() {
                            "none".into()
                        } else {
                            v.engines.join(", ")
                        }
                    )
                },
            );
            Ok(())
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let json_out = cli.json;
    match run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) if e.reported => ExitCode::from(e.exit),
        Err(e) => {
            let code = e.code.map(|c| c.code.to_string()).or(e.raw_code.clone());
            if json_out {
                println!(
                    "{}",
                    json!({"error": {"exit_code": e.exit, "code": code, "message": e.message, "hint": e.code.map(|c| c.hint_en), "etag": e.etag}})
                );
            } else {
                eprintln!("error: {}", e.message);
                if let Some(c) = e.code {
                    eprintln!("  {}", c.vi);
                    eprintln!("hint: {}", c.hint_en);
                }
                if let Some(etag) = &e.etag {
                    eprintln!("current etag: {etag}");
                }
            }
            ExitCode::from(e.exit)
        }
    }
}
