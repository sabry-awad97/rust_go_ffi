pub mod ffi;
#[cfg(feature = "auto-install")]
mod installer;

use log::{debug, info};
use semver::Version;
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
    VersionMismatch {
        expected: Version,
        found: Version,
    },
    InitializationError(String),
}

impl std::fmt::Display for DllError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DllError::NotFound => write!(f, "DLL not found"),
            DllError::LoadError(msg) => write!(f, "Failed to load DLL: {}", msg),
            #[cfg(feature = "auto-install")]
            DllError::InstallError(e) => write!(f, "Failed to install DLL: {}", e),
            DllError::VersionMismatch { expected, found } => write!(
                f,
                "Version mismatch: expected {}, found {}",
                expected, found
            ),
            DllError::InitializationError(msg) => write!(f, "Initialization error: {}", msg),
        }
    }
}

impl std::error::Error for DllError {}

pub struct DllContext {
    version: Version,
    handle: Option<winapi::shared::minwindef::HMODULE>,
    initialized: bool,
}

// Implement Send and Sync for DllContext
unsafe impl Send for DllContext {}
unsafe impl Sync for DllContext {}

impl DllContext {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for DllContext {
    fn default() -> Self {
        Self {
            version: Version::new(0, 1, 0),
            handle: None,
            initialized: false,
        }
    }
}

lazy_static::lazy_static! {
    static ref DLL_CONTEXT: parking_lot::RwLock<DllContext> = parking_lot::RwLock::new(DllContext::new());
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
    with_dll(|| {
        debug!("Calling add_numbers with {} and {}", a, b);
        let result = unsafe { AddNumbers(a as i64, b as i64) as i32 };
        debug!("add_numbers result: {}", result);
        Ok(result)
    })
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

/// Initialize the FFI system with specific version requirements
pub fn initialize(required_version: Version) -> Result<(), DllError> {
    info!("Initializing FFI system with version {}", required_version);
    let mut context = DLL_CONTEXT.write();

    if context.initialized {
        debug!("FFI system already initialized");
        let current_version = context.version.clone();
        if current_version != required_version {
            return Err(DllError::VersionMismatch {
                expected: required_version,
                found: current_version,
            });
        }
        return Ok(());
    }

    load_dll()?;

    // Get and verify version
    let dll_version = unsafe { get_dll_version() }?;
    debug!(
        "DLL version: {}, Required version: {}",
        dll_version, required_version
    );

    if dll_version != required_version {
        debug!("Version mismatch detected");
        return Err(DllError::VersionMismatch {
            expected: required_version,
            found: dll_version,
        });
    }

    context.version = dll_version;
    context.initialized = true;
    info!("FFI system initialized successfully");
    Ok(())
}

/// Cleanup FFI resources
pub fn cleanup() -> Result<(), DllError> {
    info!("Cleaning up FFI resources");
    let mut context = DLL_CONTEXT.write();

    if let Some(handle) = context.handle {
        unsafe {
            winapi::um::libloaderapi::FreeLibrary(handle);
        }
        context.handle = None;
        context.initialized = false;
    }

    Ok(())
}

// Safe wrapper with automatic initialization
pub fn with_dll<F, T>(f: F) -> Result<T, DllError>
where
    F: FnOnce() -> Result<T, DllError>,
{
    initialize(Version::new(0, 1, 0))?;
    let result = f();
    if cfg!(feature = "auto-cleanup") {
        cleanup()?;
    }
    result
}

// Add metrics and monitoring
#[cfg(feature = "metrics")]
metrics! {
    FFI_CALLS: Counter = "Number of FFI function calls made",
    FFI_ERRORS: Counter = "Number of FFI errors encountered",
    FFI_LATENCY: Histogram = "Latency of FFI calls in milliseconds"
}

#[allow(non_snake_case)]
unsafe fn get_dll_version() -> Result<Version, DllError> {
    let version_num = ffi::GetDLLVersion();
    let major = (version_num / 10000) as u64;
    let minor = ((version_num % 10000) / 100) as u64;
    let patch = (version_num % 100) as u64;

    Ok(Version::new(major, minor, patch))
}

// Safe wrapper for version checking
pub fn get_version() -> Result<Version, DllError> {
    load_dll()?;
    unsafe { get_dll_version() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test; // Add logging to tests

    #[test]
    fn test_dll_path_resolution() {
        let path = get_dll_path();
        assert!(path.is_some(), "DLL path should be resolvable");
    }

    #[test]
    fn test_dll_availability_check() {
        let available = is_dll_available();
        // This test might fail if DLL isn't present, which is expected
        println!("DLL availability: {}", available);
    }

    #[test]
    fn test_dll_verification() {
        match verify_dll() {
            Ok(_) => println!("DLL verified successfully"),
            Err(e) => println!(
                "DLL verification failed (expected if DLL not present): {:?}",
                e
            ),
        }
    }

    #[test]
    fn test_add_numbers() {
        if verify_dll().is_ok() {
            match add_numbers(5, 3) {
                Ok(result) => assert_eq!(result, 8, "Addition should work correctly"),
                Err(e) => panic!("Failed to add numbers: {:?}", e),
            }
        } else {
            println!("Skipping add_numbers test as DLL is not available");
        }
    }

    #[test]
    fn test_dll_error_display() {
        let error = DllError::NotFound;
        assert_eq!(error.to_string(), "DLL not found");

        let error = DllError::LoadError("test error".to_string());
        assert_eq!(error.to_string(), "Failed to load DLL: test error");
    }

    #[test]
    fn test_initialization() {
        initialize(Version::new(0, 1, 0)).expect("Initialization should succeed");
        assert!(DLL_CONTEXT.read().initialized);
        cleanup().expect("Cleanup should succeed");
    }

    #[test]
    fn test_version_compatibility() {
        // First initialize with correct version to ensure DLL is loaded
        initialize(Version::new(0, 1, 0)).expect("Should initialize with correct version");
        cleanup().expect("Cleanup should succeed");

        // Then try with incorrect version
        let required_version = Version::new(99, 0, 0);
        let result = initialize(required_version.clone());

        match result {
            Ok(_) => panic!("Should fail with version mismatch"),
            Err(DllError::VersionMismatch { expected, found }) => {
                assert_eq!(expected, required_version);
                assert_eq!(found, Version::new(0, 1, 0));
                println!(
                    "Successfully caught version mismatch: expected {}, found {}",
                    expected, found
                );
            }
            Err(e) => panic!("Wrong error type: {:?}", e),
        }
    }

    #[test]
    fn test_get_version() {
        match get_version() {
            Ok(version) => {
                assert_eq!(version, Version::new(0, 1, 0));
                println!("Current DLL version: {}", version);
            }
            Err(e) => panic!("Failed to get version: {:?}", e),
        }
    }

    #[test]
    fn test_version_parsing() {
        unsafe {
            let version_num = ffi::GetDLLVersion();
            assert_eq!(version_num, 100); // 0.1.0 = 100

            let version = get_dll_version().unwrap();
            assert_eq!(version.major, 0);
            assert_eq!(version.minor, 1);
            assert_eq!(version.patch, 0);
        }
    }
}
