//! Repository automation: `cargo xtask <command>`. Every CI job calls exactly one command here.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

mod guest;
mod multiuser;

const USAGE: &str = "usage: cargo xtask <command>

commands:
  ci [--quick]   fmt check, workspace lint policy, clippy -D warnings, tests (--quick skips clippy)
  e2e [filter]   run the virtaxctl end-to-end tests (optionally filter by test name)
  guest-kernel   fetch the pinned test kernel and build the busybox initramfs (.cache/guest)
  kvm-test       run the tests that boot real guests on KVM (needs /dev/kvm and guest-kernel)
  schema         write the machine.toml JSON Schema to schemas/virtax.machine.v1.json
  deny           supply-chain gate: cargo-deny (licences, bans, sources, advisories) and cargo-vet
  audit          cargo-audit against the RustSec database (run daily in CI)
  spec           OpenSpec strict validation of openspec/
  proto [ref]    buf lint for virtax.v1; with a git ref, also buf breaking against it (e.g. origin/main)
  multiuser-test prove another OS user cannot use the daemon (Linux, needs passwordless sudo)";

/// Pinned versions of the tools xtask runs through npx.
const OPENSPEC: &str = "@fission-ai/openspec@2.1.1";
const BUF: &str = "@bufbuild/buf@1.75.0";
/// Root of the protobuf module, as buf sees it (see buf.yaml).
const PROTO_ROOT: &str = "crates/virtax-api/proto";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives in the workspace")
        .to_path_buf()
}

fn cargo(args: &[&str]) -> Result<(), String> {
    eprintln!("$ cargo {}", args.join(" "));
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let status = Command::new(cargo)
        .args(args)
        .current_dir(root())
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("`cargo {}` failed", args.join(" ")))
    }
}

/// Crates allowed to use `unsafe`. They must deny undocumented unsafe blocks instead.
const UNSAFE_ALLOWED: &[&str] = &["crates/virtax-vmm-hv"];

/// Every package must inherit the workspace lints, which forbid `unsafe_code`, except the crates in
/// `UNSAFE_ALLOWED`, which must require a `SAFETY:` comment on every unsafe block.
fn check_lint_policy() -> Result<(), String> {
    let mut missing = Vec::new();
    for krate in UNSAFE_ALLOWED {
        let manifest = root().join(krate).join("Cargo.toml");
        let text = std::fs::read_to_string(&manifest)
            .map_err(|e| format!("{}: {e}", manifest.display()))?;
        if !text.contains("undocumented_unsafe_blocks = \"deny\"") {
            missing.push(format!(
                "{}: must deny clippy::undocumented_unsafe_blocks",
                manifest.display()
            ));
        }
    }
    for dir in ["crates", "apps"] {
        let Ok(entries) = std::fs::read_dir(root().join(dir)) else {
            continue;
        };
        for entry in entries.flatten() {
            let manifest = entry.path().join("Cargo.toml");
            if UNSAFE_ALLOWED.iter().any(|k| entry.path().ends_with(k)) {
                continue;
            }
            if let Ok(text) = std::fs::read_to_string(&manifest)
                && !text.contains("[lints]\nworkspace = true")
            {
                missing.push(manifest.display().to_string());
            }
        }
    }
    let ws = std::fs::read_to_string(root().join("Cargo.toml")).map_err(|e| e.to_string())?;
    if !ws.contains("unsafe_code = \"forbid\"") {
        missing.push("workspace Cargo.toml: unsafe_code must be forbid".into());
    }
    if missing.is_empty() {
        eprintln!("lint policy: ok");
        Ok(())
    } else {
        Err(format!(
            "packages not inheriting the workspace lints:\n  {}",
            missing.join("\n  ")
        ))
    }
}

fn ci(quick: bool) -> Result<(), String> {
    cargo(&["fmt", "--all", "--check"])?;
    check_lint_policy()?;
    if !quick {
        cargo(&[
            "clippy",
            "--workspace",
            "--all-targets",
            "--locked",
            "--",
            "-D",
            "warnings",
        ])?;
    }
    cargo(&["test", "--workspace", "--locked"])
}

fn e2e(args: &[String]) -> Result<(), String> {
    let mut command = vec![
        "test",
        "--locked",
        "--package",
        "virtaxctl",
        "--test",
        "e2e",
    ];
    command.push("--");
    command.extend(args.iter().map(String::as_str));
    cargo(&command)
}

fn kvm_test() -> Result<(), String> {
    let dir = guest::dir(&root());
    if !dir.join("vmlinuz").exists() || !dir.join("initramfs.cpio").exists() {
        return Err("no test guest; run `cargo xtask guest-kernel` first".into());
    }
    cargo(&["test", "--workspace", "--locked", "--", "--ignored", "kvm_"])
}

/// Run an external tool from the workspace root; a missing tool is reported with how to get it.
fn tool(program: &str, args: &[&str], install_hint: &str) -> Result<(), String> {
    eprintln!("$ {program} {}", args.join(" "));
    let status = Command::new(program)
        .args(args)
        .current_dir(root())
        .status()
        .map_err(|e| format!("{program}: {e} ({install_hint})"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("`{program} {}` failed", args.join(" ")))
    }
}

fn deny() -> Result<(), String> {
    cargo(&["deny", "--log-level", "error", "check"])?;
    cargo(&["vet", "--locked"])
}

fn spec() -> Result<(), String> {
    tool(
        "npx",
        &[
            "--yes",
            OPENSPEC,
            "validate",
            "--all",
            "--strict",
            "--no-interactive",
        ],
        "install Node.js",
    )
}

fn proto(against: Option<&str>) -> Result<(), String> {
    tool("npx", &["--yes", BUF, "lint"], "install Node.js")?;
    if let Some(git_ref) = against {
        let input = format!(".git#ref={git_ref},subdir={PROTO_ROOT}");
        tool(
            "npx",
            &["--yes", BUF, "breaking", "--against", &input],
            "install Node.js",
        )?;
    }
    Ok(())
}

fn schema() -> Result<(), String> {
    let out = root().join("schemas");
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let output = Command::new(cargo)
        .args([
            "run",
            "--quiet",
            "--package",
            "virtaxctl",
            "--",
            "vm",
            "schema",
        ])
        .current_dir(root())
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    let path = out.join("virtax.machine.v1.json");
    std::fs::write(&path, output.stdout).map_err(|e| e.to_string())?;
    eprintln!("wrote {}", path.display());
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("ci") => ci(args.iter().any(|a| a == "--quick")),
        Some("e2e") => e2e(&args[1..]),
        Some("schema") => schema(),
        Some("guest-kernel") => guest::fetch(&root()),
        Some("kvm-test") => kvm_test(),
        Some("deny") => deny(),
        Some("audit") => cargo(&["audit"]),
        Some("spec") => spec(),
        Some("proto") => proto(args.get(1).map(String::as_str)),
        Some("multiuser-test") => multiuser::run_test(&root()),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
