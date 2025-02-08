use serde::ser::Error;
use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::string::FromUtf8Error;
use std::{thread, time::Duration};

// Constants
const DEFAULT_RUST_VERSION: &str = "1.70";
const GO_LIB_DIR: &str = "go_lib";
const GO_SOURCE: &str = "go_lib/go_lib.go";
const DLL_FILE: &str = "go_lib/go_lib.dll";
const HEADER_FILE: &str = "go_lib/go_lib.h";

// Traits for different build steps
trait Builder {
    fn build(&self) -> Result<(), BuildError>;
}

trait WindowsBuilder: Builder {
    fn generate_def_file(&self) -> Result<(), BuildError>;
    fn generate_lib_file(&self) -> Result<(), BuildError>;
    fn copy_dll(&self) -> Result<(), BuildError>;
}

// Error handling
#[derive(Debug)]
pub enum BuildError {
    GoBuild {
        cmd: String,
        source: io::Error,
    },
    DumpbinExecution {
        cmd: String,
        source: io::Error,
    },
    DumpbinFailed {
        output: Vec<u8>,
    },
    DlltoolExecution {
        cmd: String,
        source: io::Error,
    },
    DlltoolFailed {
        output: Vec<u8>,
    },
    FileOperation {
        path: String,
        operation: String,
        source: io::Error,
    },
    MetadataExecution {
        cmd: String,
        source: io::Error,
    },
    MetadataParsing {
        source: serde_json::Error,
    },
    ConfigParsing {
        source: toml::de::Error,
    },
    Utf8Conversion {
        source: FromUtf8Error,
    },
    EnvVar {
        var: String,
        source: std::env::VarError,
    },
    BindgenFailed,
}

impl From<io::Error> for BuildError {
    fn from(error: io::Error) -> Self {
        BuildError::FileOperation {
            path: String::new(),
            operation: "access".to_string(),
            source: error,
        }
    }
}

impl From<FromUtf8Error> for BuildError {
    fn from(error: FromUtf8Error) -> Self {
        BuildError::Utf8Conversion { source: error }
    }
}

impl From<serde_json::Error> for BuildError {
    fn from(error: serde_json::Error) -> Self {
        BuildError::MetadataParsing { source: error }
    }
}

impl From<toml::de::Error> for BuildError {
    fn from(error: toml::de::Error) -> Self {
        BuildError::ConfigParsing { source: error }
    }
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildError::GoBuild { cmd, source } => {
                write!(
                    f,
                    "Failed to execute Go build command '{}': {}",
                    cmd, source
                )
            }
            BuildError::DumpbinExecution { cmd, source } => {
                write!(f, "Failed to execute dumpbin command '{}': {}", cmd, source)
            }
            BuildError::DumpbinFailed { output } => {
                write!(
                    f,
                    "Dumpbin command failed: {}",
                    String::from_utf8_lossy(output)
                )
            }
            BuildError::DlltoolExecution { cmd, source } => {
                write!(f, "Failed to execute dlltool command '{}': {}", cmd, source)
            }
            BuildError::DlltoolFailed { output } => {
                write!(
                    f,
                    "Dlltool command failed: {}",
                    String::from_utf8_lossy(output)
                )
            }
            BuildError::FileOperation {
                path,
                operation,
                source,
            } => {
                write!(f, "Failed to {} file '{}': {}", operation, path, source)
            }
            BuildError::MetadataExecution { cmd, source } => {
                write!(
                    f,
                    "Failed to execute cargo metadata command '{}': {}",
                    cmd, source
                )
            }
            BuildError::MetadataParsing { source } => {
                write!(f, "Failed to parse cargo metadata: {}", source)
            }
            BuildError::ConfigParsing { source } => {
                write!(f, "Failed to parse Cargo.toml: {}", source)
            }
            BuildError::Utf8Conversion { source } => {
                write!(f, "Failed to convert output to UTF-8: {}", source)
            }
            BuildError::EnvVar { var, source } => {
                write!(
                    f,
                    "Failed to read environment variable '{}': {}",
                    var, source
                )
            }
            BuildError::BindgenFailed => todo!(),
        }
    }
}

impl std::error::Error for BuildError {}

// Add this function to the file
fn generate_def_content(dumpbin_output: &[u8]) -> Result<String, BuildError> {
    let dumpbin_stdout = String::from_utf8(dumpbin_output.to_vec())
        .map_err(|e| BuildError::Utf8Conversion { source: e })?;

    let mut def_content = String::from("EXPORTS\n");

    for line in dumpbin_stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if let Some(function_name) = extract_function_name(&parts) {
            def_content.push_str(&format!("{}\n", function_name));
        }
    }

    Ok(def_content)
}

fn extract_function_name<'a>(parts: &'a [&'a str]) -> Option<&'a str> {
    if parts.len() > 3 && parts[0].parse::<u32>().is_ok() {
        Some(parts[3])
    } else {
        None
    }
}

// Also add this missing function for parsing cargo metadata
fn parse_cargo_metadata(metadata_output: &[u8]) -> Result<PathBuf, BuildError> {
    let metadata: serde_json::Value = serde_json::from_slice(metadata_output)?;
    let target_directory =
        metadata["target_directory"]
            .as_str()
            .ok_or_else(|| BuildError::MetadataParsing {
                source: serde_json::Error::custom("target_directory not found"),
            })?;

    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    Ok(PathBuf::from(target_directory).join(profile))
}

// Configuration struct
#[derive(Clone)]
struct BuildConfig {
    go_source: String,
    dll_file: String,
    header_file: String,
    target_dir: PathBuf,
    rust_version: String,
}

impl BuildConfig {
    fn new() -> Result<Self, BuildError> {
        Ok(Self {
            go_source: GO_SOURCE.to_string(),
            dll_file: DLL_FILE.to_string(),
            header_file: HEADER_FILE.to_string(),
            target_dir: get_target_directory()?,
            rust_version: get_rust_version()?,
        })
    }
}

// Implementation for Go library builder
struct GoLibBuilder {
    config: BuildConfig,
}

impl GoLibBuilder {
    fn new(config: BuildConfig) -> Self {
        Self { config }
    }
}

impl Builder for GoLibBuilder {
    fn build(&self) -> Result<(), BuildError> {
        let output = Command::new("go")
            .args([
                "build",
                "-buildmode=c-shared",
                "-o",
                &self.config.dll_file,
                &self.config.go_source,
            ])
            .output()
            .map_err(|e| BuildError::GoBuild {
                cmd: "go build".to_string(),
                source: e,
            })?;

        if !output.status.success() {
            return Err(BuildError::GoBuild {
                cmd: "go build".to_string(),
                source: std::io::Error::new(
                    std::io::ErrorKind::Other,
                    String::from_utf8_lossy(&output.stderr).to_string(),
                ),
            });
        }

        Ok(())
    }
}

// Windows-specific implementation
struct WindowsGoLibBuilder {
    config: BuildConfig,
}

impl WindowsGoLibBuilder {
    fn new(config: BuildConfig) -> Self {
        Self { config }
    }
}

impl Builder for WindowsGoLibBuilder {
    fn build(&self) -> Result<(), BuildError> {
        self.generate_def_file()?;
        self.generate_lib_file()?;
        self.copy_dll()?;
        Ok(())
    }
}

impl WindowsBuilder for WindowsGoLibBuilder {
    fn generate_def_file(&self) -> Result<(), BuildError> {
        let def_file = format!("{}/go_lib.def", GO_LIB_DIR);
        let dumpbin_output = Command::new("dumpbin")
            .args(["/exports", &self.config.dll_file])
            .output()
            .map_err(|e| BuildError::DumpbinExecution {
                cmd: "dumpbin".to_string(),
                source: e,
            })?;

        if !dumpbin_output.status.success() {
            return Err(BuildError::DumpbinFailed {
                output: dumpbin_output.stderr,
            });
        }

        let def_content = generate_def_content(&dumpbin_output.stdout)?;
        fs::write(&def_file, def_content).map_err(|e| BuildError::FileOperation {
            path: self.config.dll_file.clone(),
            operation: "copy".to_string(),
            source: e,
        })?;

        Ok(())
    }

    fn generate_lib_file(&self) -> Result<(), BuildError> {
        let def_file = format!("{}/go_lib.def", GO_LIB_DIR);
        let lib_file = format!("{}/go_lib.lib", GO_LIB_DIR);

        let dlltool_output = Command::new("dlltool")
            .args([
                "-d",
                &def_file,
                "-D",
                &self.config.dll_file,
                "-l",
                &lib_file,
            ])
            .output()
            .map_err(|e| BuildError::DlltoolExecution {
                cmd: "dlltool".to_string(),
                source: e,
            })?;

        if !dlltool_output.status.success() {
            return Err(BuildError::DlltoolFailed {
                output: dlltool_output.stderr,
            });
        }

        Ok(())
    }

    fn copy_dll(&self) -> Result<(), BuildError> {
        let target_dll_path = self.config.target_dir.join("go_lib.dll");
        copy_with_retry(&PathBuf::from(&self.config.dll_file), &target_dll_path, 5).map_err(
            |e| BuildError::FileOperation {
                path: self.config.dll_file.clone(),
                operation: "copy".to_string(),
                source: e,
            },
        )?;
        Ok(())
    }
}

// Build orchestrator
#[derive(Clone)]
struct BuildOrchestrator {
    config: BuildConfig,
}

impl BuildOrchestrator {
    fn new() -> Result<Self, BuildError> {
        Ok(Self {
            config: BuildConfig::new()?,
        })
    }

    fn run(&self) -> Result<(), BuildError> {
        // Build Go library
        let go_builder = GoLibBuilder::new(self.config.clone());
        go_builder.build()?;

        // Windows-specific processing
        if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
            let windows_builder = WindowsGoLibBuilder::new(self.config.clone());
            windows_builder.build()?;
        }

        // Generate bindings
        self.generate_bindings()?;
        self.set_cargo_config();

        Ok(())
    }

    fn generate_bindings(&self) -> Result<(), BuildError> {
        let out_dir = env::var("OUT_DIR").unwrap();
        let bindings_path = Path::new(&out_dir).join("bindings.rs");

        let bindings = bindgen::Builder::default()
            .rust_target(self.config.rust_version.parse().unwrap())
            .header(&self.config.header_file)
            .generate()
            .map_err(|_| BuildError::BindgenFailed)?;

        bindings
            .write_to_file(bindings_path.as_path())
            .map_err(|e| BuildError::FileOperation {
                path: bindings_path.to_string_lossy().to_string(),
                operation: "write".to_string(),
                source: e,
            })?;

        Ok(())
    }

    fn set_cargo_config(&self) {
        println!("cargo:rustc-link-search=native={}", GO_LIB_DIR);
        println!("cargo:rustc-link-lib=dylib={}", GO_LIB_DIR);
        println!("cargo:rerun-if-changed={}", self.config.header_file);
        println!("cargo:rerun-if-changed={}", self.config.go_source);
    }
}

// Helper functions
fn get_target_directory() -> Result<PathBuf, BuildError> {
    if let Ok(target_dir) = env::var("CARGO_TARGET_DIR") {
        return Ok(PathBuf::from(target_dir).join("debug"));
    }

    let output = Command::new("cargo")
        .args(["metadata", "--format-version=1", "--no-deps"])
        .output()
        .map_err(|e| BuildError::MetadataExecution {
            cmd: "cargo metadata".to_string(),
            source: e,
        })?;

    if !output.status.success() {
        return Err(BuildError::MetadataExecution {
            cmd: "cargo metadata".to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::Other,
                String::from_utf8_lossy(&output.stderr).to_string(),
            ),
        });
    }

    // Parse metadata and return target directory
    parse_cargo_metadata(&output.stdout)
}

pub fn get_rust_version() -> Result<String, BuildError> {
    let cargo_toml = fs::read_to_string("Cargo.toml").map_err(|e| BuildError::FileOperation {
        path: "Cargo.toml".to_string(),
        operation: "read".to_string(),
        source: e,
    })?;

    let parsed_toml: toml::Value = toml::from_str(&cargo_toml)?;

    Ok(parsed_toml
        .get("package")
        .and_then(|package| package.get("rust-version"))
        .and_then(|version| version.as_str())
        .unwrap_or(DEFAULT_RUST_VERSION)
        .to_string())
}

fn copy_with_retry(src: &Path, dst: &Path, retries: u32) -> io::Result<()> {
    for attempt in 1..=retries {
        match fs::copy(src, dst) {
            Ok(_) => return Ok(()),
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied && attempt < retries => {
                println!(
                    "File locked, retrying in 1 second (attempt {}/{})",
                    attempt, retries
                );
                thread::sleep(Duration::from_secs(1));
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::Other,
        "Failed to copy file after all retries",
    ))
}

fn build_go_lib() -> Result<(), BuildError> {
    println!("Starting Go library initialization...");

    // Create go_lib directory if it doesn't exist
    fs::create_dir_all("go_lib").map_err(|e| BuildError::FileOperation {
        path: "go_lib".to_string(),
        operation: "create_dir".to_string(),
        source: e,
    })?;

    // Check if go.mod already exists
    if !Path::new("go_lib/go.mod").exists() {
        // Initialize Go module
        let init_output = Command::new("go")
            .args(["mod", "init", "go_lib"])
            .current_dir("go_lib")
            .output()
            .map_err(|e| BuildError::GoBuild {
                cmd: "go mod init".to_string(),
                source: e,
            })?;

        if !init_output.status.success() {
            return Err(BuildError::GoBuild {
                cmd: "go mod init".to_string(),
                source: io::Error::new(
                    io::ErrorKind::Other,
                    String::from_utf8_lossy(&init_output.stderr).to_string(),
                ),
            });
        }

        // Add required dependency
        let get_output = Command::new("go")
            .args(["get", "modernc.org/sqlite@latest"])
            .current_dir("go_lib")
            .output()
            .map_err(|e| BuildError::GoBuild {
                cmd: "go get".to_string(),
                source: e,
            })?;

        if !get_output.status.success() {
            return Err(BuildError::GoBuild {
                cmd: "go get".to_string(),
                source: io::Error::new(
                    io::ErrorKind::Other,
                    String::from_utf8_lossy(&get_output.stderr).to_string(),
                ),
            });
        }
    }

    // Download dependencies
    let tidy_output = Command::new("go")
        .args(["mod", "tidy"])
        .current_dir("go_lib")
        .output()
        .map_err(|e| BuildError::GoBuild {
            cmd: "go mod tidy".to_string(),
            source: e,
        })?;

    if !tidy_output.status.success() {
        return Err(BuildError::GoBuild {
            cmd: "go mod tidy".to_string(),
            source: io::Error::new(
                io::ErrorKind::Other,
                String::from_utf8_lossy(&tidy_output.stderr).to_string(),
            ),
        });
    }

    // Clean up existing DLL if possible
    let dll_path = PathBuf::from("go_lib/go_lib.dll");
    if dll_path.exists() {
        if let Err(e) = fs::remove_file(&dll_path) {
            println!("Warning: Could not remove existing DLL: {}", e);
        }
    }

    // Build Go code
    let output = Command::new("go")
        .args(["build", "-buildmode=c-shared", "-o", "go_lib.dll"])
        .current_dir("go_lib")
        .output()
        .map_err(|e| BuildError::GoBuild {
            cmd: "go build".to_string(),
            source: e,
        })?;

    if !output.status.success() {
        return Err(BuildError::GoBuild {
            cmd: "go build".to_string(),
            source: io::Error::new(
                io::ErrorKind::Other,
                String::from_utf8_lossy(&output.stderr).to_string(),
            ),
        });
    }

    // Copy with retry mechanism
    copy_with_retry(
        &PathBuf::from("go_lib/go_lib.dll"),
        &PathBuf::from("target/debug/go_lib.dll"),
        5,
    )
    .map_err(|e| BuildError::FileOperation {
        path: "go_lib/go_lib.dll".to_string(),
        operation: "copy".to_string(),
        source: e,
    })?;

    Ok(())
}

// Main function
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Rust-Go binding build process...");

    // First, build Go library with retry mechanism
    build_go_lib()?;

    let orchestrator = BuildOrchestrator::new().expect("Failed to create build orchestrator");

    println!("Building Go shared library...");
    if let Err(e) = orchestrator.run() {
        panic!("Build failed: {:?}", e);
    }
    println!("Build completed successfully!");

    // Verify DLL path
    let dll_path = Path::new(DLL_FILE);
    if dll_path.exists() {
        println!("DLL file found at: {:?}", dll_path);
    } else {
        panic!("DLL file not found at: {:?}", dll_path);
    }

    // Generate bindings
    orchestrator.generate_bindings()?;

    Ok(())
}
