#[cfg(target_os = "windows")]
use crate::platform::PermissionState;
#[cfg(target_os = "windows")]
use crate::{platform::PermissionStatus, storage::files::now_ms};

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsIntegrityLevel {
    Low,
    Medium,
    High,
    System,
    Unknown,
}

#[cfg(target_os = "windows")]
impl WindowsIntegrityLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            WindowsIntegrityLevel::Low => "low",
            WindowsIntegrityLevel::Medium => "medium",
            WindowsIntegrityLevel::High => "high",
            WindowsIntegrityLevel::System => "system",
            WindowsIntegrityLevel::Unknown => "unknown",
        }
    }
}

#[cfg(target_os = "windows")]
pub fn settings_label() -> &'static str {
    "Windows Input Control"
}

#[cfg(target_os = "windows")]
pub fn input_status() -> PermissionState {
    // Low-level mouse hooks and SendInput work for normal desktop apps.
    // UIPI can still block elevated windows, which S1 reports as a limitation.
    PermissionState::Granted
}

#[cfg(target_os = "windows")]
pub fn permission_status() -> PermissionStatus {
    let windows_input = input_status();
    PermissionStatus {
        accessibility: PermissionState::Unsupported,
        input_monitoring: PermissionState::Unsupported,
        screen_recording: PermissionState::Unsupported,
        windows_input,
        windows_integrity_level: integrity_level().ok().map(|level| level.as_str().into()),
        can_capture_mouse: windows_input == PermissionState::Granted,
        can_inject_mouse: windows_input == PermissionState::Granted,
        updated_at_ms: now_ms(),
    }
}

#[cfg(target_os = "windows")]
pub fn integrity_level() -> Result<WindowsIntegrityLevel, String> {
    use std::mem::size_of;

    use windows::Win32::{
        Foundation::{CloseHandle, HANDLE},
        Security::{
            GetSidSubAuthority, GetSidSubAuthorityCount, GetTokenInformation, TokenIntegrityLevel,
            TOKEN_MANDATORY_LABEL, TOKEN_QUERY,
        },
        System::{
            SystemServices::{
                SECURITY_MANDATORY_HIGH_RID, SECURITY_MANDATORY_LOW_RID,
                SECURITY_MANDATORY_MEDIUM_RID, SECURITY_MANDATORY_SYSTEM_RID,
            },
            Threading::{GetCurrentProcess, OpenProcessToken},
        },
    };

    let mut token = HANDLE::default();
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) }
        .map_err(|err| format!("OpenProcessToken failed: {err}"))?;

    let result = (|| {
        let mut needed = 0u32;
        let _ = unsafe { GetTokenInformation(token, TokenIntegrityLevel, None, 0, &mut needed) };
        if needed < size_of::<TOKEN_MANDATORY_LABEL>() as u32 {
            return Err("TokenIntegrityLevel returned no buffer size".into());
        }

        let mut buffer = vec![0u8; needed as usize];
        unsafe {
            GetTokenInformation(
                token,
                TokenIntegrityLevel,
                Some(buffer.as_mut_ptr().cast()),
                needed,
                &mut needed,
            )
        }
        .map_err(|err| format!("GetTokenInformation(TokenIntegrityLevel) failed: {err}"))?;

        let label = unsafe { &*(buffer.as_ptr() as *const TOKEN_MANDATORY_LABEL) };
        let sid = label.Label.Sid;
        let count_ptr = unsafe { GetSidSubAuthorityCount(sid) };
        if count_ptr.is_null() {
            return Err("integrity SID has no sub-authority count".into());
        }
        let count = unsafe { *count_ptr };
        if count == 0 {
            return Err("integrity SID has zero sub-authorities".into());
        }
        let rid_ptr = unsafe { GetSidSubAuthority(sid, u32::from(count - 1)) };
        if rid_ptr.is_null() {
            return Err("integrity SID has no RID".into());
        }
        let rid = unsafe { *rid_ptr };
        Ok(classify_integrity_rid(
            rid,
            SECURITY_MANDATORY_LOW_RID as u32,
            SECURITY_MANDATORY_MEDIUM_RID as u32,
            SECURITY_MANDATORY_HIGH_RID as u32,
            SECURITY_MANDATORY_SYSTEM_RID as u32,
        ))
    })();

    let _ = unsafe { CloseHandle(token) };
    result
}

#[cfg(target_os = "windows")]
fn classify_integrity_rid(
    rid: u32,
    low_rid: u32,
    medium_rid: u32,
    high_rid: u32,
    system_rid: u32,
) -> WindowsIntegrityLevel {
    if rid >= system_rid {
        WindowsIntegrityLevel::System
    } else if rid >= high_rid {
        WindowsIntegrityLevel::High
    } else if rid >= medium_rid {
        WindowsIntegrityLevel::Medium
    } else if rid >= low_rid {
        WindowsIntegrityLevel::Low
    } else {
        WindowsIntegrityLevel::Unknown
    }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;
    use windows::Win32::System::SystemServices::{
        SECURITY_MANDATORY_HIGH_RID, SECURITY_MANDATORY_LOW_RID, SECURITY_MANDATORY_MEDIUM_RID,
        SECURITY_MANDATORY_SYSTEM_RID,
    };

    #[test]
    fn classifies_integrity_rids() {
        let low = SECURITY_MANDATORY_LOW_RID as u32;
        let medium = SECURITY_MANDATORY_MEDIUM_RID as u32;
        let high = SECURITY_MANDATORY_HIGH_RID as u32;
        let system = SECURITY_MANDATORY_SYSTEM_RID as u32;

        assert_eq!(
            classify_integrity_rid(low, low, medium, high, system),
            WindowsIntegrityLevel::Low
        );
        assert_eq!(
            classify_integrity_rid(medium, low, medium, high, system),
            WindowsIntegrityLevel::Medium
        );
        assert_eq!(
            classify_integrity_rid(high, low, medium, high, system),
            WindowsIntegrityLevel::High
        );
        assert_eq!(
            classify_integrity_rid(system, low, medium, high, system),
            WindowsIntegrityLevel::System
        );
        assert_eq!(
            classify_integrity_rid(low - 1, low, medium, high, system),
            WindowsIntegrityLevel::Unknown
        );
    }
}
