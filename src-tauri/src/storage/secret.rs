use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::identity::PrivateKeyRef;

const DEFAULT_PRIVATE_KEY_FILE: &str = "identity.key";

#[derive(Debug, Error)]
pub enum SecretStoreError {
    #[error("io error: {0}")]
    Io(String),
    #[error("invalid private key path")]
    InvalidPath,
}

pub trait SecretStore {
    fn load_private_key(&self) -> Result<Vec<u8>, SecretStoreError>;
    fn save_private_key(&self, key: &[u8]) -> Result<PrivateKeyRef, SecretStoreError>;
}

#[derive(Debug, Clone)]
pub struct FileSecretStore {
    base_dir: PathBuf,
    key_file_name: String,
}

impl FileSecretStore {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            key_file_name: DEFAULT_PRIVATE_KEY_FILE.into(),
        }
    }

    pub fn with_file_name(base_dir: PathBuf, key_file_name: impl Into<String>) -> Self {
        Self {
            base_dir,
            key_file_name: key_file_name.into(),
        }
    }

    pub fn key_path(&self) -> PathBuf {
        self.base_dir.join(&self.key_file_name)
    }

    fn key_ref(&self) -> PrivateKeyRef {
        PrivateKeyRef::FileEncrypted {
            path: self.key_file_name.clone(),
        }
    }
}

impl SecretStore for FileSecretStore {
    fn load_private_key(&self) -> Result<Vec<u8>, SecretStoreError> {
        fs::read(self.key_path()).map_err(|err| SecretStoreError::Io(err.to_string()))
    }

    fn save_private_key(&self, key: &[u8]) -> Result<PrivateKeyRef, SecretStoreError> {
        fs::create_dir_all(&self.base_dir).map_err(|err| SecretStoreError::Io(err.to_string()))?;
        let key_path = self.key_path();
        write_secret_atomic(&key_path, key)?;
        harden_private_key_file(&key_path)?;
        Ok(self.key_ref())
    }
}

fn write_secret_atomic(path: &Path, key: &[u8]) -> Result<(), SecretStoreError> {
    let parent = path.parent().ok_or(SecretStoreError::InvalidPath)?;
    fs::create_dir_all(parent).map_err(|err| SecretStoreError::Io(err.to_string()))?;

    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or(SecretStoreError::InvalidPath)?;
    let tmp_path = path.with_file_name(format!("{file_name}.tmp"));
    match fs::remove_file(&tmp_path) {
        Ok(()) => {}
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => return Err(SecretStoreError::Io(err.to_string())),
    }

    fs::write(&tmp_path, key).map_err(|err| SecretStoreError::Io(err.to_string()))?;
    fs::rename(&tmp_path, path).map_err(|err| SecretStoreError::Io(err.to_string()))
}

#[cfg(unix)]
fn harden_private_key_file(path: &Path) -> Result<(), SecretStoreError> {
    use std::os::unix::fs::PermissionsExt;

    let permissions = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, permissions).map_err(|err| SecretStoreError::Io(err.to_string()))
}

#[cfg(windows)]
fn harden_private_key_file(path: &Path) -> Result<(), SecretStoreError> {
    windows_acl::set_owner_only_acl(path)
}

#[cfg(not(any(unix, windows)))]
fn harden_private_key_file(_path: &Path) -> Result<(), SecretStoreError> {
    Ok(())
}

#[cfg(windows)]
mod windows_acl {
    use std::{ffi::c_void, mem::size_of, os::windows::ffi::OsStrExt, path::Path, ptr};

    use windows::{
        core::{PCWSTR, PWSTR},
        Win32::{
            Foundation::{CloseHandle, GetLastError, HANDLE, HLOCAL, WIN32_ERROR},
            Security::{
                Authorization::{
                    SetEntriesInAclW, SetNamedSecurityInfoW, EXPLICIT_ACCESS_W, SET_ACCESS,
                    SE_FILE_OBJECT, TRUSTEE_IS_SID, TRUSTEE_IS_USER, TRUSTEE_W,
                },
                GetTokenInformation, TokenOwner, DACL_SECURITY_INFORMATION,
                OWNER_SECURITY_INFORMATION, PROTECTED_DACL_SECURITY_INFORMATION, PSID, TOKEN_OWNER,
                TOKEN_QUERY,
            },
            Storage::FileSystem::FILE_ALL_ACCESS,
            System::Threading::{GetCurrentProcess, OpenProcessToken},
        },
    };

    use super::SecretStoreError;

    pub fn set_owner_only_acl(path: &Path) -> Result<(), SecretStoreError> {
        let owner = current_owner_sid()?;
        let owner_sid = PSID(owner.as_ptr() as *mut c_void);
        let trustee = TRUSTEE_W {
            TrusteeForm: TRUSTEE_IS_SID,
            TrusteeType: TRUSTEE_IS_USER,
            ptstrName: PWSTR(owner_sid.0.cast()),
            ..Default::default()
        };
        let access = EXPLICIT_ACCESS_W {
            grfAccessPermissions: FILE_ALL_ACCESS.0,
            grfAccessMode: SET_ACCESS,
            Trustee: trustee,
            ..Default::default()
        };
        let mut dacl = ptr::null_mut();
        let status = unsafe { SetEntriesInAclW(Some(&[access]), None, &mut dacl) };
        if status != WIN32_ERROR(0) {
            return Err(win32_error("SetEntriesInAclW", status));
        }

        let path_wide = path_to_wide(path);
        let status = unsafe {
            SetNamedSecurityInfoW(
                PCWSTR(path_wide.as_ptr()),
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION
                    | DACL_SECURITY_INFORMATION
                    | PROTECTED_DACL_SECURITY_INFORMATION,
                owner_sid,
                None,
                Some(dacl),
                None,
            )
        };
        unsafe {
            windows::Win32::Foundation::LocalFree(HLOCAL(dacl.cast()));
        }
        if status != WIN32_ERROR(0) {
            return Err(win32_error("SetNamedSecurityInfoW", status));
        }

        Ok(())
    }

    fn current_owner_sid() -> Result<Vec<u8>, SecretStoreError> {
        let mut token = HANDLE::default();
        unsafe {
            OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)
                .map_err(|err| SecretStoreError::Io(format!("OpenProcessToken failed: {err}")))?;
        }
        let token_guard = TokenHandle(token);

        let mut required = 0;
        let _ = unsafe { GetTokenInformation(token_guard.0, TokenOwner, None, 0, &mut required) };
        if required == 0 {
            let err = unsafe { GetLastError() };
            return Err(win32_error("GetTokenInformation size", err));
        }

        let mut buffer = vec![0_u8; required as usize];
        unsafe {
            GetTokenInformation(
                token_guard.0,
                TokenOwner,
                Some(buffer.as_mut_ptr() as *mut c_void),
                required,
                &mut required,
            )
            .map_err(|err| SecretStoreError::Io(format!("GetTokenInformation failed: {err}")))?;
        }
        let owner = unsafe { *(buffer.as_ptr() as *const TOKEN_OWNER) };
        let sid_len = unsafe { windows::Win32::Security::GetLengthSid(owner.Owner) };
        let mut sid = vec![0_u8; sid_len as usize];
        unsafe {
            windows::Win32::Security::CopySid(sid_len, PSID(sid.as_mut_ptr().cast()), owner.Owner)
                .map_err(|err| SecretStoreError::Io(format!("CopySid failed: {err}")))?;
        }

        Ok(sid)
    }

    fn path_to_wide(path: &Path) -> Vec<u16> {
        path.as_os_str().encode_wide().chain([0]).collect()
    }

    fn win32_error(context: &str, status: WIN32_ERROR) -> SecretStoreError {
        SecretStoreError::Io(format!("{context} failed with Win32 error {}", status.0))
    }

    struct TokenHandle(HANDLE);

    impl Drop for TokenHandle {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }

    const _: () = {
        assert!(size_of::<TOKEN_OWNER>() >= size_of::<PSID>());
    };
}
