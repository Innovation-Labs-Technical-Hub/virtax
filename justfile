# Virtax developer recipes. `just` lists them; `make <recipe>` forwards here.

set shell := ["bash", "-euo", "pipefail", "-c"]

root := justfile_directory()
bin := root / "target" / "debug"
guest := root / ".cache" / "guest"
# The dev daemon keeps its data and pidfile here, separate from a real installation.
dev := root / ".cache" / "dev"
# Unix socket paths must stay short (about 100 bytes), so the socket lives in the runtime dir.
socket := env_var_or_default("XDG_RUNTIME_DIR", "/tmp") / "virtax-dev" / "virtaxd.sock"
ctl := bin / "virtaxctl" + " --socket " + socket

# List recipes.
default:
    @just --list --unsorted

# --- Toolchain -----------------------------------------------------------------------------

# Show the pinned toolchain and tool versions.
toolchain:
    rustc --version
    cargo --version
    just --version
    @echo "workspace: $(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; print(", ".join(sorted(p["name"] for p in json.load(sys.stdin)["packages"])))')"

# Check what this host can run: KVM access, busybox-static, the test guest.
doctor:
    @if [ -r /dev/kvm ] && [ -w /dev/kvm ]; then echo "kvm:      ok (/dev/kvm read-write)"; else echo "kvm:      unavailable (need /dev/kvm read-write, e.g. the kvm group)"; fi
    @if file -L /usr/bin/busybox 2>/dev/null | grep -q 'statically linked'; then echo "busybox:  ok (static)"; else echo "busybox:  missing or dynamic (install busybox-static)"; fi
    @if [ -f "{{guest}}/vmlinuz" ] && [ -f "{{guest}}/initramfs.cpio" ]; then echo "guest:    ok ({{guest}})"; else echo "guest:    missing (run: just guest)"; fi
    @if [ -f "{{dev}}/virtaxd.pid" ] && kill -0 "$(cat {{dev}}/virtaxd.pid)" 2>/dev/null; then echo "daemon:   running (pid $(cat {{dev}}/virtaxd.pid))"; else echo "daemon:   stopped (run: just dev-up)"; fi

# --- Build and check -----------------------------------------------------------------------

# Build every crate and binary (debug).
build:
    cargo build --workspace

# Build optimized binaries.
release:
    cargo build --workspace --release

# Format the code.
fmt:
    cargo fmt --all

# Clippy with warnings as errors.
lint:
    cargo clippy --workspace --all-targets --locked -- -D warnings

# Unit and end-to-end tests (no KVM needed). Pass a filter to narrow, e.g. `just test events`.
test *filter:
    cargo test --workspace --locked -- {{filter}}

# Everything CI runs: fmt check, lint policy, clippy, tests.
ci:
    cargo xtask ci

# End-to-end tests through the real virtaxd and virtaxctl binaries.
e2e *filter:
    cargo xtask e2e {{filter}}

# Validate the OpenSpec specs and changes (strict), e.g. before merging a change.
spec:
    cargo xtask spec

# Licences, advisories, bans and sources (cargo-deny), then audited dependencies (cargo-vet).
deny:
    cargo xtask deny

# RustSec advisories for Cargo.lock (cargo-audit).
audit:
    cargo xtask audit

# buf lint on the protocol; with a git ref, also buf breaking against it, e.g. `just proto origin/main`.
proto *against:
    cargo xtask proto {{against}}

# Regenerate schemas/virtax.machine.v1.json.
schema:
    cargo xtask schema

# --- Guests on KVM -------------------------------------------------------------------------

# Fetch the pinned test kernel and build the busybox initramfs into .cache/guest.
guest:
    cargo xtask guest-kernel

# Tests that boot real guests on KVM.
kvm-test: guest
    cargo xtask kvm-test

# Boot the test guest directly with virtax-vmm, e.g. `just boot 4 512 virtax.hold` (Ctrl-C stops it).
boot cpus="2" memory="512" cmdline="": guest
    @cargo build -q -p virtax-vmm
    @{{bin}}/virtax-vmm --kernel {{guest}}/vmlinuz --initrd {{guest}}/initramfs.cpio \
        --cpus {{cpus}} --memory-mib {{memory}} --cmdline "{{cmdline}}"

# --- Dev daemon ----------------------------------------------------------------------------

# Start the dev daemon in the background (MockEngine plus virtax-vmm when KVM is usable).
dev-up: build
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p "{{dev}}"
    if [ -f "{{dev}}/virtaxd.pid" ] && kill -0 "$(cat "{{dev}}/virtaxd.pid")" 2>/dev/null; then
        echo "virtaxd already running (pid $(cat "{{dev}}/virtaxd.pid")), socket {{socket}}"
        exit 0
    fi
    "{{bin}}/virtaxd" --mock-engine --socket "{{socket}}" --data-dir "{{dev}}/data" \
        --vmm-bin "{{bin}}/virtax-vmm" >>"{{dev}}/virtaxd.log" 2>&1 &
    echo $! > "{{dev}}/virtaxd.pid"
    for _ in $(seq 50); do [ -S "{{socket}}" ] && break; sleep 0.1; done
    if [ ! -S "{{socket}}" ]; then echo "virtaxd did not start; see {{dev}}/virtaxd.log"; tail -5 "{{dev}}/virtaxd.log"; exit 1; fi
    echo "virtaxd running (pid $(cat "{{dev}}/virtaxd.pid")), socket {{socket}}, log {{dev}}/virtaxd.log"
    {{ctl}} version

# Stop the dev daemon (its guests stop with it).
dev-down:
    #!/usr/bin/env bash
    set -euo pipefail
    pidfile="{{dev}}/virtaxd.pid"
    if [ -f "$pidfile" ] && kill -0 "$(cat "$pidfile")" 2>/dev/null; then
        kill "$(cat "$pidfile")"
        for _ in $(seq 70); do kill -0 "$(cat "$pidfile")" 2>/dev/null || break; sleep 0.1; done
        echo "virtaxd stopped"
    else
        echo "virtaxd is not running"
    fi
    rm -f "$pidfile"

# Follow the dev daemon log.
dev-log:
    tail -f "{{dev}}/virtaxd.log"

# Delete the dev daemon's data (machines, registry, logs). Stops the daemon first.
dev-reset: dev-down
    rm -rf "{{dev}}"

# Run virtaxctl against the dev daemon, e.g. `just ctl vm list` or `just ctl --json vm show demo`.
ctl *args:
    @{{ctl}} {{args}}

# Watch the dev daemon's events (NDJSON).
events *args:
    @{{ctl}} --json events watch {{args}}

# Create and start a KVM guest named `demo` on the dev daemon, then show its console log.
demo name="demo" cpus="2": guest dev-up
    #!/usr/bin/env bash
    set -euo pipefail
    if ! {{ctl}} vm show "{{name}}" >/dev/null 2>&1; then
        {{ctl}} vm create --name "{{name}}" --engine virtax-vmm --cpus "{{cpus}}" --memory 512 \
            --kernel "{{guest}}/vmlinuz" --initrd "{{guest}}/initramfs.cpio" --cmdline virtax.hold
    fi
    {{ctl}} vm start "{{name}}"
    folder="$({{ctl}} --json vm show "{{name}}" | python3 -c 'import json,sys; print(json.load(sys.stdin)["folder"])')"
    for _ in $(seq 50); do grep -q VIRTAX-GUEST-OK "$folder/logs/serial.log" 2>/dev/null && break; sleep 0.1; done
    grep VIRTAX-GUEST "$folder/logs/serial.log" | tail -3
    echo "running; shut it down with: just ctl vm stop {{name}}"

# Show a machine's serial console log, e.g. `just serial demo`.
serial name="demo":
    @tail -n 40 "$({{ctl}} --json vm show "{{name}}" | python3 -c 'import json,sys; print(json.load(sys.stdin)["folder"])')/logs/serial.log"
