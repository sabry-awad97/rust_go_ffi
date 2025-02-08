use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// Constants
const DEFAULT_RUST_VERSION: &str = "1.70";
const GO_LIB_DIR: &str = "go_lib";

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
enum BuildError {
    GoBuildFailed(String),
    DumpbinFailed(String),
    DlltoolFailed(String),
    FileOperationFailed(String),
    MetadataError(String),
    ConfigError(String),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::GoBuildFailed(e) => write!(f, "Go build failed: {}", e),
            BuildError::DumpbinFailed(e) => write!(f, "Dumpbin failed: {}", e),
            BuildError::DlltoolFailed(e) => write!(f, "Dlltool failed: {}", e),
            BuildError::FileOperationFailed(e) => write!(f, "File operation failed: {}", e),
            BuildError::MetadataError(e) => write!(f, "Metadata error: {}", e),
            BuildError::ConfigError(e) => write!(f, "Config error: {}", e),
        }
    }
}

impl std::error::Error for BuildError {}

// Add this function to the file
fn generate_def_content(dumpbin_output: &[u8]) -> Result<String, BuildError> {
    let dumpbin_stdout = String::from_utf8(dumpbin_output.to_vec())
        .map_err(|e| BuildError::DumpbinFailed(e.to_string()))?;

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
    let metadata: serde_json::Value = serde_json::from_slice(metadata_output)
        .map_err(|e| BuildError::MetadataError(e.to_string()))?;

    let target_directory = metadata["target_directory"]
        .as_str()
        .ok_or_else(|| BuildError::MetadataError("target_directory not found".to_string()))?;

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
            go_source: format!("{}/go_lib.go", GO_LIB_DIR),
            dll_file: format!("{}/go_lib.dll", GO_LIB_DIR),
            header_file: format!("{}/go_lib.h", GO_LIB_DIR),
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
            .map_err(|e| BuildError::GoBuildFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(BuildError::GoBuildFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
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
            .map_err(|e| BuildError::DumpbinFailed(e.to_string()))?;

        if !dumpbin_output.status.success() {
            return Err(BuildError::DumpbinFailed(
                String::from_utf8_lossy(&dumpbin_output.stderr).to_string(),
            ));
        }

        let def_content = generate_def_content(&dumpbin_output.stdout)?;
        fs::write(&def_file, def_content)
            .map_err(|e| BuildError::FileOperationFailed(e.to_string()))?;

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
            .map_err(|e| BuildError::DlltoolFailed(e.to_string()))?;

        if !dlltool_output.status.success() {
            return Err(BuildError::DlltoolFailed(
                String::from_utf8_lossy(&dlltool_output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    fn copy_dll(&self) -> Result<(), BuildError> {
        let target_dll_path = self.config.target_dir.join("go_lib.dll");
        fs::copy(&self.config.dll_file, target_dll_path)
            .map_err(|e| BuildError::FileOperationFailed(e.to_string()))?;
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
            .map_err(|_| BuildError::ConfigError("Failed to generate bindings".to_string()))?;

        bindings
            .write_to_file(bindings_path)
            .map_err(|e| BuildError::FileOperationFailed(e.to_string()))?;

        Ok(())
    }

    fn set_cargo_config(&self) {
        println!("cargo:rustc-link-search=native={}", GO_LIB_DIR);
        println!("cargo:rustc-link-lib=dylib=go_lib");
        println!("cargo:rerun-if-changed={}/go_lib.h", GO_LIB_DIR);
        println!("cargo:rerun-if-changed={}/go_lib.go", GO_LIB_DIR);
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
        .map_err(|e| BuildError::MetadataError(e.to_string()))?;

    if !output.status.success() {
        return Err(BuildError::MetadataError(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    // Parse metadata and return target directory
    parse_cargo_metadata(&output.stdout)
}

fn get_rust_version() -> Result<String, BuildError> {
    let cargo_toml =
        fs::read_to_string("Cargo.toml").map_err(|e| BuildError::ConfigError(e.to_string()))?;

    let parsed_toml: toml::Value =
        toml::from_str(&cargo_toml).map_err(|e| BuildError::ConfigError(e.to_string()))?;

    Ok(parsed_toml
        .get("package")
        .and_then(|package| package.get("rust-version"))
        .and_then(|version| version.as_str())
        .unwrap_or(DEFAULT_RUST_VERSION)
        .to_string())
}

// Main function
fn main() {
    let orchestrator = BuildOrchestrator::new().expect("Failed to create build orchestrator");
    if let Err(e) = orchestrator.run() {
        panic!("Build failed: {:?}", e);
    }
}
