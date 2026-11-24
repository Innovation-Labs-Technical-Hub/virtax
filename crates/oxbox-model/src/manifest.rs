//! The portable per-VM manifest `machine.toml`, schema `oxbox.machine/1`.
//!
//! Engine-specific escape hatches live under `[engine_overrides.<engine>]` rather than `[engine.*]`
//! because TOML cannot hold both `engine = "<kind>"` and an `[engine]` table. Top-level keys that
//! start with `x-` are extensions: they are kept verbatim across a read/write round trip.

use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const SCHEMA_ID: &str = "oxbox.machine/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum EngineKind {
    /// VirtualBox 7.2 driven through VBoxCAPI (Phase 1).
    VboxLegacy,
    /// The Rust VMM (Phase 2).
    Oxvmm,
    /// Deterministic in-process engine for development and tests; never runs a guest.
    Mock,
}

impl EngineKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::VboxLegacy => "vbox-legacy",
            Self::Oxvmm => "oxvmm",
            Self::Mock => "mock",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "vbox-legacy" => Some(Self::VboxLegacy),
            "oxvmm" => Some(Self::Oxvmm),
            "mock" => Some(Self::Mock),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Arch {
    X86_64,
    Aarch64,
}

impl Arch {
    pub fn host() -> Self {
        if cfg!(target_arch = "aarch64") {
            Self::Aarch64
        } else {
            Self::X86_64
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64",
            Self::Aarch64 => "aarch64",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Chipset {
    Piix3,
    Ich9,
    Q35,
    Armv8Virtual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Firmware {
    Bios,
    Efi,
}

impl Firmware {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "bios" => Some(Self::Bios),
            "efi" => Some(Self::Efi),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bios => "bios",
            Self::Efi => "efi",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub enum Tpm {
    #[default]
    #[serde(rename = "none")]
    None,
    #[serde(rename = "1.2")]
    V1_2,
    #[serde(rename = "2.0")]
    V2_0,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Platform {
    pub arch: Arch,
    pub chipset: Chipset,
    pub firmware: Firmware,
    #[serde(default)]
    pub secure_boot: bool,
    #[serde(default)]
    pub tpm: Tpm,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Cpu {
    pub count: u32,
    /// Scheduling hint, e.g. `performance-cores`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Memory {
    pub size_mib: u64,
    #[serde(default)]
    pub balloon: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Display {
    #[serde(default = "Display::default_device")]
    pub device: String,
    #[serde(default = "Display::default_vram")]
    pub vram_mib: u32,
    #[serde(default = "Display::default_monitors")]
    pub monitors: u8,
}

impl Display {
    fn default_device() -> String {
        "vmsvga".into()
    }
    fn default_vram() -> u32 {
        16
    }
    fn default_monitors() -> u8 {
        1
    }
}

impl Default for Display {
    fn default() -> Self {
        Self {
            device: Self::default_device(),
            vram_mib: Self::default_vram(),
            monitors: Self::default_monitors(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Bus {
    Ide,
    Sata,
    Scsi,
    Nvme,
    Virtio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DeviceKind {
    Disk,
    Dvd,
    Floppy,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Attachment {
    pub port: u8,
    #[serde(default)]
    pub device: u8,
    pub kind: DeviceKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub medium: Option<Uuid>,
    /// Path relative to the machine folder, or absolute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default)]
    pub ssd: bool,
    #[serde(default)]
    pub discard: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StorageController {
    pub name: String,
    pub bus: Bus,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Storage {
    #[serde(default)]
    pub controllers: Vec<StorageController>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum NicModel {
    E1000,
    VirtioNet,
    Pcnet,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum NetAttachment {
    /// Forwards use `proto:host_port:guest_port`, e.g. `tcp:2222:22`.
    Nat {
        #[serde(default)]
        forwards: Vec<String>,
    },
    Bridged {
        interface: String,
    },
    HostOnly {
        network: String,
    },
    Internal {
        network: String,
    },
    NotAttached,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Nic {
    pub slot: u8,
    pub model: NicModel,
    pub mac: String,
    pub attachment: NetAttachment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Clipboard {
    #[default]
    Disabled,
    HostToGuest,
    GuestToHost,
    Bidirectional,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SharedFolder {
    pub name: String,
    pub host: String,
    pub guest: String,
    #[serde(default)]
    pub rw: bool,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Integration {
    #[serde(default)]
    pub clipboard: Clipboard,
    #[serde(default)]
    pub shared_folders: Vec<SharedFolder>,
}

/// Top-level keys of the manifest; anything else must be an `x-` extension.
const KNOWN_KEYS: &[&str] = &[
    "schema",
    "id",
    "name",
    "os_type",
    "groups",
    "tags",
    "engine",
    "platform",
    "cpu",
    "memory",
    "display",
    "storage",
    "nics",
    "integration",
    "engine_overrides",
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MachineManifest {
    pub schema: String,
    pub id: Uuid,
    pub name: String,
    pub os_type: String,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub engine: EngineKind,
    pub platform: Platform,
    pub cpu: Cpu,
    pub memory: Memory,
    #[serde(default)]
    pub display: Display,
    #[serde(default)]
    pub storage: Storage,
    #[serde(default)]
    pub nics: Vec<Nic>,
    #[serde(default)]
    pub integration: Integration,
    /// Per-engine key/value escape hatch, e.g. `[engine_overrides.vbox-legacy]`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub engine_overrides: BTreeMap<String, BTreeMap<String, String>>,
    /// Top-level `x-*` keys, preserved verbatim.
    #[serde(skip)]
    #[schemars(skip)]
    pub extensions: BTreeMap<String, toml::Value>,
}

/// A validation or parse problem, located by field path and (when parsed from text) line/column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Issue {
    pub path: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
}

impl Issue {
    fn at(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
            line: None,
            column: None,
        }
    }
}

impl std::fmt::Display for Issue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.line, self.column) {
            (Some(l), Some(c)) => write!(f, "{l}:{c}: {}: {}", self.path, self.message),
            _ => write!(f, "{}: {}", self.path, self.message),
        }
    }
}

fn line_col(text: &str, offset: usize) -> (usize, usize) {
    let before = &text[..offset.min(text.len())];
    let line = before.matches('\n').count() + 1;
    let col = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
    (line, col)
}

/// Parse and validate manifest text. On failure, every issue found is returned.
pub fn parse_manifest(text: &str) -> Result<MachineManifest, Vec<Issue>> {
    let table: toml::Table = toml::from_str(text).map_err(|e| vec![toml_issue(text, &e)])?;
    let mut manifest: MachineManifest =
        toml::from_str(text).map_err(|e| vec![toml_issue(text, &e)])?;

    let mut issues = Vec::new();
    for (key, value) in table {
        if key.starts_with("x-") {
            manifest.extensions.insert(key, value);
        } else if !KNOWN_KEYS.contains(&key.as_str()) {
            let mut issue = Issue::at(&key, "unknown key (extensions must start with `x-`)");
            if let Some(pos) = text.find(&format!("{key} ")).or_else(|| text.find(&key)) {
                let (l, c) = line_col(text, pos);
                issue.line = Some(l);
                issue.column = Some(c);
            }
            issues.push(issue);
        }
    }
    issues.extend(manifest.validate());
    if issues.is_empty() {
        Ok(manifest)
    } else {
        Err(issues)
    }
}

fn toml_issue(text: &str, e: &toml::de::Error) -> Issue {
    let mut issue = Issue::at("", e.message().trim());
    if let Some(span) = e.span() {
        let (l, c) = line_col(text, span.start);
        issue.line = Some(l);
        issue.column = Some(c);
    }
    issue
}

fn valid_mac(mac: &str) -> bool {
    let parts: Vec<&str> = mac.split(':').collect();
    parts.len() == 6
        && parts
            .iter()
            .all(|p| p.len() == 2 && u8::from_str_radix(p, 16).is_ok())
}

impl MachineManifest {
    /// A new manifest with defaults suited to the host architecture.
    pub fn new(name: impl Into<String>, os_type: impl Into<String>, engine: EngineKind) -> Self {
        let arch = Arch::host();
        Self {
            schema: SCHEMA_ID.into(),
            id: Uuid::new_v4(),
            name: name.into(),
            os_type: os_type.into(),
            groups: Vec::new(),
            tags: Vec::new(),
            engine,
            platform: Platform {
                arch,
                chipset: if arch == Arch::Aarch64 {
                    Chipset::Armv8Virtual
                } else {
                    Chipset::Q35
                },
                firmware: Firmware::Efi,
                secure_boot: false,
                tpm: Tpm::None,
            },
            cpu: Cpu {
                count: 2,
                prefer: None,
            },
            memory: Memory {
                size_mib: 2048,
                balloon: false,
            },
            display: Display::default(),
            storage: Storage::default(),
            nics: Vec::new(),
            integration: Integration::default(),
            engine_overrides: BTreeMap::new(),
            extensions: BTreeMap::new(),
        }
    }

    /// Semantic checks that the type system cannot express.
    pub fn validate(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        if self.schema != SCHEMA_ID {
            issues.push(Issue::at(
                "schema",
                format!("expected `{SCHEMA_ID}`, got `{}`", self.schema),
            ));
        }
        if self.name.trim().is_empty() {
            issues.push(Issue::at("name", "must not be empty"));
        } else if self
            .name
            .chars()
            .any(|c| c.is_control() || c == '/' || c == '\\')
        {
            issues.push(Issue::at(
                "name",
                "must not contain control characters or path separators",
            ));
        }
        if self.os_type.trim().is_empty() {
            issues.push(Issue::at("os_type", "must not be empty"));
        }
        for (i, g) in self.groups.iter().enumerate() {
            if !g.starts_with('/') {
                issues.push(Issue::at(
                    format!("groups[{i}]"),
                    "group paths start with `/`",
                ));
            }
        }
        if !(1..=256).contains(&self.cpu.count) {
            issues.push(Issue::at("cpu.count", "must be between 1 and 256"));
        }
        if self.memory.size_mib < 4 {
            issues.push(Issue::at("memory.size_mib", "must be at least 4 MiB"));
        }
        if !(1..=256).contains(&self.display.vram_mib) {
            issues.push(Issue::at("display.vram_mib", "must be between 1 and 256"));
        }
        if !(1..=8).contains(&self.display.monitors) {
            issues.push(Issue::at("display.monitors", "must be between 1 and 8"));
        }
        if self.platform.secure_boot && self.platform.firmware != Firmware::Efi {
            issues.push(Issue::at(
                "platform.secure_boot",
                "requires `firmware = \"efi\"`",
            ));
        }
        let mut names = BTreeSet::new();
        for (i, c) in self.storage.controllers.iter().enumerate() {
            if !names.insert(c.name.as_str()) {
                issues.push(Issue::at(
                    format!("storage.controllers[{i}].name"),
                    "duplicate controller name",
                ));
            }
            let mut slots = BTreeSet::new();
            for (j, a) in c.attachments.iter().enumerate() {
                if !slots.insert((a.port, a.device)) {
                    issues.push(Issue::at(
                        format!("storage.controllers[{i}].attachments[{j}]"),
                        "port/device already used on this controller",
                    ));
                }
            }
        }
        let mut slots = BTreeSet::new();
        for (i, n) in self.nics.iter().enumerate() {
            if !slots.insert(n.slot) {
                issues.push(Issue::at(format!("nics[{i}].slot"), "duplicate NIC slot"));
            }
            if !valid_mac(&n.mac) {
                issues.push(Issue::at(
                    format!("nics[{i}].mac"),
                    "expected six hex pairs like 08:00:27:AB:12:CD",
                ));
            }
        }
        issues
    }

    /// Serialize to TOML, re-inserting `x-*` extensions.
    pub fn to_toml(&self) -> String {
        let mut table = toml::Table::try_from(self).expect("manifest serializes to a TOML table");
        for (k, v) in &self.extensions {
            table.insert(k.clone(), v.clone());
        }
        toml::to_string_pretty(&table).expect("TOML table serializes")
    }
}

/// JSON Schema for `oxbox.machine/1`, generated from the serde types.
pub fn json_schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(MachineManifest)).expect("schema serializes")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
schema = "oxbox.machine/1"
id = "6b0f2c1e-8a4d-4c2e-9a77-1f3b5e2d0c11"
name = "ubuntu-24.04"
os_type = "Ubuntu_64"
groups = ["/Dev lab"]
engine = "vbox-legacy"
x-team = { owner = "infra" }

[platform]
arch = "x86_64"
chipset = "q35"
firmware = "efi"
secure_boot = true
tpm = "2.0"

[cpu]
count = 4
prefer = "performance-cores"

[memory]
size_mib = 8192
balloon = true

[[storage.controllers]]
name = "nvme0"
bus = "nvme"
  [[storage.controllers.attachments]]
  port = 0
  kind = "disk"
  path = "disks/disk.qcow2"
  ssd = true

[[nics]]
slot = 0
model = "virtio-net"
mac = "08:00:27:AB:12:CD"
attachment = { kind = "nat", forwards = ["tcp:2222:22"] }

[engine_overrides.vbox-legacy]
"VBoxInternal/TM/TSCMode" = "RealTSCOffset"
"#;

    #[test]
    fn parses_sample_and_round_trips_extensions() {
        let m = parse_manifest(SAMPLE).expect("valid");
        assert_eq!(m.cpu.count, 4);
        assert_eq!(m.platform.tpm, Tpm::V2_0);
        assert!(m.extensions.contains_key("x-team"));
        let again = parse_manifest(&m.to_toml()).expect("round trip");
        assert_eq!(m, again);
    }

    #[test]
    fn reports_line_and_column_for_type_errors() {
        let bad = SAMPLE.replace("count = 4", "count = \"four\"");
        let issues = parse_manifest(&bad).unwrap_err();
        let line = bad.lines().position(|l| l.contains("\"four\"")).unwrap() + 1;
        assert_eq!(issues[0].line, Some(line));
        assert!(issues[0].column.is_some());
    }

    #[test]
    fn rejects_unknown_top_level_keys_but_keeps_x_keys() {
        let bad = SAMPLE.replace("x-team", "team");
        let issues = parse_manifest(&bad).unwrap_err();
        assert!(issues.iter().any(|i| i.path == "team" && i.line.is_some()));
    }

    #[test]
    fn rejects_unknown_nested_keys() {
        let bad = SAMPLE.replace("balloon = true", "baloon = true");
        assert!(parse_manifest(&bad).is_err());
    }

    #[test]
    fn semantic_validation() {
        let mut m = MachineManifest::new("vm", "Linux_64", EngineKind::Mock);
        assert!(m.validate().is_empty());
        m.cpu.count = 0;
        m.platform.firmware = Firmware::Bios;
        m.platform.secure_boot = true;
        let paths: Vec<_> = m.validate().into_iter().map(|i| i.path).collect();
        assert!(paths.contains(&"cpu.count".to_string()));
        assert!(paths.contains(&"platform.secure_boot".to_string()));
    }

    #[test]
    fn schema_has_required_fields() {
        let schema = json_schema();
        let required = schema["required"].as_array().unwrap();
        assert!(required.iter().any(|v| v == "engine"));
        assert!(schema["properties"].get("extensions").is_none());
    }
}
