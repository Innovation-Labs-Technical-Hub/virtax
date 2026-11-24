//! OXE error catalogue. Every code carries an English and a Vietnamese sentence plus a fix hint,
//! so clients never show raw engine codes.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OxeCode {
    pub code: &'static str,
    pub en: &'static str,
    pub vi: &'static str,
    pub hint_en: &'static str,
}

macro_rules! catalogue {
    ($($name:ident = $code:literal, $en:literal, $vi:literal, $hint:literal;)*) => {
        impl OxeCode {
            $(pub const $name: OxeCode = OxeCode { code: $code, en: $en, vi: $vi, hint_en: $hint };)*
            pub const ALL: &'static [OxeCode] = &[$(OxeCode::$name),*];
        }
    };
}

catalogue! {
    CFG_INVALID_MANIFEST = "OXE-CFG-010", "The machine manifest is invalid.",
        "Tệp cấu hình máy ảo không hợp lệ.", "Run `oxctl vm validate <path>` to see each error with its line and column.";
    CFG_ETAG_MISMATCH = "OXE-CFG-011", "The machine was changed by someone else since you loaded it.",
        "Máy ảo đã bị thay đổi bởi phiên khác kể từ khi bạn tải.", "Reload the machine and apply your change again.";
    CFG_MANIFEST_RECOVERED = "OXE-CFG-012", "The manifest was damaged and was restored from its previous copy.",
        "Tệp cấu hình bị hỏng và đã được khôi phục từ bản trước.", "Review the machine settings; the damaged file was kept as machine.toml.corrupt-<time>.";
    CFG_MANIFEST_UNREADABLE = "OXE-CFG-013", "The manifest cannot be read and no usable previous copy exists.",
        "Không đọc được tệp cấu hình và không có bản sao trước dùng được.", "Fix or restore machine.toml in the machine folder, then restart oxboxd.";
    CFG_MIGRATION_FAILED = "OXE-CFG-021", "Upgrading the registry failed; the previous version was kept.",
        "Nâng cấp registry thất bại; phiên bản cũ được giữ nguyên.", "Send the oxboxd log to support; the registry was not modified.";
    CFG_REGISTRY_TOO_NEW = "OXE-CFG-022", "The registry was written by a newer OxBox version.",
        "Registry được tạo bởi phiên bản OxBox mới hơn.", "Update OxBox; an older daemon never writes to a newer registry.";
    CFG_VM_LOCKED = "OXE-CFG-030", "The machine is in use by another OxBox daemon.",
        "Máy ảo đang được một tiến trình OxBox khác sử dụng.", "Close OxBox on the other account or host that holds the machine.";
    CFG_NAME_TAKEN = "OXE-CFG-031", "A machine with this name already exists.",
        "Đã có máy ảo trùng tên.", "Choose a different name.";
    API_NOT_FOUND = "OXE-API-404", "The requested resource does not exist.",
        "Tài nguyên được yêu cầu không tồn tại.", "Check the name with `oxctl vm list`.";
    API_PERMISSION_DENIED = "OXE-API-403", "Only the user who owns this daemon may connect.",
        "Chỉ người dùng sở hữu daemon này mới được kết nối.", "Run the client as the same user as oxboxd.";
    API_BAD_REQUEST = "OXE-API-400", "The request is malformed.",
        "Yêu cầu không hợp lệ.", "See `oxctl --help` for the accepted arguments.";
    API_DAEMON_RUNNING = "OXE-API-409", "Another oxboxd is already running for this user.",
        "Đã có một oxboxd khác đang chạy cho người dùng này.", "Use the running daemon, or stop it with `oxctl daemon stop`.";
    API_RESYNC_REQUIRED = "OXE-API-410", "The event cursor is older than the retained event history.",
        "Con trỏ sự kiện cũ hơn lịch sử sự kiện còn lưu.", "Reload the full state, then watch again without --since.";
    RT_INVALID_TRANSITION = "OXE-RT-001", "The machine cannot do this in its current state.",
        "Máy ảo không thể thực hiện thao tác này ở trạng thái hiện tại.", "Wait for the current operation to finish, or stop the machine first.";
    RT_NEEDS_POWER_OFF = "OXE-RT-002", "Some changes require the machine to be powered off.",
        "Một số thay đổi yêu cầu tắt máy ảo.", "Power the machine off, then apply the change again.";
    RT_ALREADY_IN_STATE = "OXE-RT-003", "The machine is already in the requested state.",
        "Máy ảo đã ở trạng thái được yêu cầu.", "No action needed.";
    RT_CANCELLED = "OXE-RT-004", "The operation was cancelled.",
        "Thao tác đã bị huỷ.", "Run the command again if you still need it.";
    ENG_NOT_AVAILABLE = "OXE-ENG-002", "The engine for this machine is not available on this host.",
        "Engine của máy ảo này không có trên máy chủ.", "Install the required engine or move the machine to an available one.";
    ENG_FAILED = "OXE-ENG-999", "The virtualization engine reported an error.",
        "Engine ảo hoá báo lỗi.", "Copy the technical details and check the machine log.";
}

impl OxeCode {
    pub fn by_code(code: &str) -> Option<OxeCode> {
        Self::ALL.iter().copied().find(|c| c.code == code)
    }
}

impl fmt::Display for OxeCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.en)
    }
}

/// An error tagged with an OXE code and an optional technical detail.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{code}{}", detail.as_ref().map(|d| format!(" ({d})")).unwrap_or_default())]
pub struct OxeError {
    pub code: OxeCode,
    pub detail: Option<String>,
}

impl OxeError {
    pub fn new(code: OxeCode) -> Self {
        Self { code, detail: None }
    }

    pub fn with(code: OxeCode, detail: impl Into<String>) -> Self {
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
        for c in OxeCode::ALL {
            assert!(seen.insert(c.code), "duplicate {}", c.code);
            assert!(c.code.starts_with("OXE-"));
            assert!(!c.en.is_empty() && !c.vi.is_empty() && !c.hint_en.is_empty());
        }
    }
}
