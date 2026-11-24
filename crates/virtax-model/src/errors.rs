//! VTX error catalogue. Every code carries an English and a Vietnamese sentence plus a fix hint,
//! so clients never show raw engine codes.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VtxCode {
    pub code: &'static str,
    pub en: &'static str,
    pub vi: &'static str,
    pub hint_en: &'static str,
}

macro_rules! catalogue {
    ($($name:ident = $code:literal, $en:literal, $vi:literal, $hint:literal;)*) => {
        impl VtxCode {
            $(pub const $name: VtxCode = VtxCode { code: $code, en: $en, vi: $vi, hint_en: $hint };)*
            pub const ALL: &'static [VtxCode] = &[$(VtxCode::$name),*];
        }
    };
}

catalogue! {
    CFG_INVALID_MANIFEST = "VTX-CFG-010", "The machine manifest is invalid.",
        "Tệp cấu hình máy ảo không hợp lệ.", "Run `virtaxctl vm validate <path>` to see each error with its line and column.";
    CFG_ETAG_MISMATCH = "VTX-CFG-011", "The machine was changed by someone else since you loaded it.",
        "Máy ảo đã bị thay đổi bởi phiên khác kể từ khi bạn tải.", "Reload the machine and apply your change again.";
    CFG_MANIFEST_RECOVERED = "VTX-CFG-012", "The manifest was damaged and was restored from its previous copy.",
        "Tệp cấu hình bị hỏng và đã được khôi phục từ bản trước.", "Review the machine settings; the damaged file was kept as machine.toml.corrupt-<time>.";
    CFG_MANIFEST_UNREADABLE = "VTX-CFG-013", "The manifest cannot be read and no usable previous copy exists.",
        "Không đọc được tệp cấu hình và không có bản sao trước dùng được.", "Fix or restore machine.toml in the machine folder, then restart virtaxd.";
    CFG_MIGRATION_FAILED = "VTX-CFG-021", "Upgrading the registry failed; the previous version was kept.",
        "Nâng cấp registry thất bại; phiên bản cũ được giữ nguyên.", "Send the virtaxd log to support; the registry was not modified.";
    CFG_REGISTRY_TOO_NEW = "VTX-CFG-022", "The registry was written by a newer Virtax version.",
        "Registry được tạo bởi phiên bản Virtax mới hơn.", "Update Virtax; an older daemon never writes to a newer registry.";
    CFG_VM_LOCKED = "VTX-CFG-030", "The machine is in use by another Virtax daemon.",
        "Máy ảo đang được một tiến trình Virtax khác sử dụng.", "Close Virtax on the other account or host that holds the machine.";
    CFG_NAME_TAKEN = "VTX-CFG-031", "A machine with this name already exists.",
        "Đã có máy ảo trùng tên.", "Choose a different name.";
    API_NOT_FOUND = "VTX-API-404", "The requested resource does not exist.",
        "Tài nguyên được yêu cầu không tồn tại.", "Check the name with `virtaxctl vm list`.";
    API_PERMISSION_DENIED = "VTX-API-403", "Only the user who owns this daemon may connect.",
        "Chỉ người dùng sở hữu daemon này mới được kết nối.", "Run the client as the same user as virtaxd.";
    API_BAD_REQUEST = "VTX-API-400", "The request is malformed.",
        "Yêu cầu không hợp lệ.", "See `virtaxctl --help` for the accepted arguments.";
    API_DAEMON_RUNNING = "VTX-API-409", "Another virtaxd is already running for this user.",
        "Đã có một virtaxd khác đang chạy cho người dùng này.", "Use the running daemon, or stop it with `virtaxctl daemon stop`.";
    API_RESYNC_REQUIRED = "VTX-API-410", "The event cursor is older than the retained event history.",
        "Con trỏ sự kiện cũ hơn lịch sử sự kiện còn lưu.", "Reload the full state, then watch again without --since.";
    RT_INVALID_TRANSITION = "VTX-RT-001", "The machine cannot do this in its current state.",
        "Máy ảo không thể thực hiện thao tác này ở trạng thái hiện tại.", "Wait for the current operation to finish, or stop the machine first.";
    RT_NEEDS_POWER_OFF = "VTX-RT-002", "Some changes require the machine to be powered off.",
        "Một số thay đổi yêu cầu tắt máy ảo.", "Power the machine off, then apply the change again.";
    RT_ALREADY_IN_STATE = "VTX-RT-003", "The machine is already in the requested state.",
        "Máy ảo đã ở trạng thái được yêu cầu.", "No action needed.";
    RT_CANCELLED = "VTX-RT-004", "The operation was cancelled.",
        "Thao tác đã bị huỷ.", "Run the command again if you still need it.";
    ENG_NOT_AVAILABLE = "VTX-ENG-002", "The engine for this machine is not available on this host.",
        "Engine của máy ảo này không có trên máy chủ.", "Install the required engine or move the machine to an available one.";
    ENG_UNSUPPORTED = "VTX-ENG-010", "This engine cannot do that yet.",
        "Engine này chưa hỗ trợ thao tác đó.", "For virtax-vmm machines, power off with `virtaxctl vm stop --force`.";
    ENG_VM_CONFIG = "VTX-ENG-011", "The machine configuration cannot run on this engine.",
        "Cấu hình máy ảo không chạy được trên engine này.", "See the detail, change the setting with `virtaxctl vm set`, then start again.";
    ENG_FAILED = "VTX-ENG-999", "The virtualization engine reported an error.",
        "Engine ảo hoá báo lỗi.", "Copy the technical details and check the machine log.";
}

impl VtxCode {
    pub fn by_code(code: &str) -> Option<VtxCode> {
        Self::ALL.iter().copied().find(|c| c.code == code)
    }
}

impl fmt::Display for VtxCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.en)
    }
}

/// An error tagged with an VTX code and an optional technical detail.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{code}{}", detail.as_ref().map(|d| format!(" ({d})")).unwrap_or_default())]
pub struct VtxError {
    pub code: VtxCode,
    pub detail: Option<String>,
}

impl VtxError {
    pub fn new(code: VtxCode) -> Self {
        Self { code, detail: None }
    }

    pub fn with(code: VtxCode, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: Some(detail.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn catalogue_codes_are_unique_and_bilingual() {
        let mut seen = HashSet::new();
        for c in VtxCode::ALL {
            assert!(seen.insert(c.code), "duplicate {}", c.code);
            assert!(c.code.starts_with("VTX-"));
            assert!(!c.en.is_empty() && !c.vi.is_empty() && !c.hint_en.is_empty());
        }
    }
}
