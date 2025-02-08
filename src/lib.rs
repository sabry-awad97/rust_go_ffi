pub mod ffi;
#[cfg(feature = "auto-install")]
mod installer;

use std::path::{Path, PathBuf};
use std::sync::Once;
static INIT: Once = Once::new();
static mut DLL_HANDLE: Option<winapi::shared::minwindef::HMODULE> = None;

/// Error type for DLL operations
#[derive(Debug)]
pub enum DllError {
    NotFound,
    LoadError(String),
    #[cfg(feature = "auto-install")]
    InstallError(std::io::Error),
}

impl std::fmt::Display for DllError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DllError::NotFound => write!(f, "DLL not found"),
            DllError::LoadError(msg) => write!(f, "Failed to load DLL: {}", msg),
            #[cfg(feature = "auto-install")]
            DllError::InstallError(e) => write!(f, "Failed to install DLL: {}", e),
        }
    }
}

/// Checks if the DLL is available in the system
pub fn is_dll_available() -> bool {
    get_dll_path().map_or(false, |path| path.exists())
}

/// Gets the path to the DLL
pub fn get_dll_path() -> Option<PathBuf> {
    #[cfg(feature = "auto-install")]
    {
        if let Ok(installer) = installer::Installer::new() {
            return Some(installer.get_dll_path());
        }
    }

    // Look in common locations
    let locations = vec![
        Path::new("go_lib/go_lib.dll"), // Local directory
        Path::new("./go_lib.dll"),      // Current directory
    ];

    locations
        .into_iter()
        .find(|p| p.exists())
        .map(PathBuf::from)
}

pub fn load_dll() -> Result<(), DllError> {
    let dll_path = get_dll_path().ok_or(DllError::NotFound)?;

    #[cfg(windows)]
    unsafe {
        INIT.call_once(|| {
            use std::os::windows::ffi::OsStrExt;
            use winapi::um::libloaderapi::LoadLibraryW;

            let wide_path: Vec<u16> = dll_path
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            let handle = LoadLibraryW(wide_path.as_ptr());
            if !handle.is_null() {
                DLL_HANDLE = Some(handle);
            }
        });

        match DLL_HANDLE {
            Some(_) => Ok(()),
            None => Err(DllError::LoadError("Failed to load DLL".to_string())),
        }
    }
}

// Modify verify_dll to use the new loading mechanism
pub fn verify_dll() -> Result<(), DllError> {
    load_dll()
}

// Re-export FFI functions with safety wrapper
pub fn add_numbers(a: i32, b: i32) -> Result<i32, DllError> {
    load_dll()?;
    Ok(unsafe { AddNumbers(a as i64, b as i64) as i32 })
}

pub fn go_function() -> Result<(), DllError> {
    load_dll()?;
    unsafe { GoFunction() };
    Ok(())
}

#[cfg(feature = "auto-install")]
/// Install the DLL if the auto-install feature is enabled
pub fn install_dll() -> Result<(), DllError> {
    let installer = installer::Installer::new().map_err(DllError::InstallError)?;

    installer.install().map_err(DllError::InstallError)?;

    installer
        .verify_installation()
        .map_err(DllError::InstallError)
}

// Keep the unsafe FFI exports but mark them as deprecated
#[deprecated(note = "Use the safe wrapper `add_numbers` instead")]
pub use ffi::AddNumbers;
#[deprecated(note = "Use the safe wrapper `go_function` instead")]
pub use ffi::GoFunction;
