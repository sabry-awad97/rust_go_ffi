extern crate bindgen;

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let go_lib_dir = "go_lib"; // Directory containing Go files
    let go_source = format!("{}/go_lib.go", go_lib_dir);
    let dll_file = format!("{}/go_lib.dll", go_lib_dir);
    let header_file = format!("{}/go_lib.h", go_lib_dir);
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = get_target_directory();
    let bindings_path = Path::new(&out_dir).join("bindings.rs");
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    // Build the Go shared library
    let output = Command::new("go")
        .args(["build", "-buildmode=c-shared", "-o", &dll_file, &go_source])
        .output()
        .expect("Failed to execute go build");

    if !output.status.success() {
        panic!(
            "Failed to build Go code: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Windows-specific processing
    if target_os == "windows" {
        let def_file = format!("{}/go_lib.def", go_lib_dir);
        let lib_file = format!("{}/go_lib.lib", go_lib_dir);

        // Generate .def file using dumpbin
        let dumpbin_output = Command::new("dumpbin")
            .args(["/exports", &dll_file])
            .output()
            .expect("Failed to run dumpbin");

        if !dumpbin_output.status.success() {
            panic!(
                "dumpbin failed: {}",
                String::from_utf8_lossy(&dumpbin_output.stderr)
            );
        }

        // Extract function names and write to .def file
        let dumpbin_stdout = String::from_utf8_lossy(&dumpbin_output.stdout);
        let mut def_content = "EXPORTS\n".to_string();
        for line in dumpbin_stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 3 && parts[0].parse::<u32>().is_ok() {
                def_content.push_str(&format!("{}\n", parts[3]));
            }
        }
        fs::write(&def_file, def_content).expect("Failed to write .def file");

        // Generate .lib using dlltool
        let dlltool_output = Command::new("dlltool")
            .args(["-d", &def_file, "-D", &dll_file, "-l", &lib_file])
            .output()
            .expect("Failed to run dlltool");

        if !dlltool_output.status.success() {
            panic!(
                "dlltool failed: {}",
                String::from_utf8_lossy(&dlltool_output.stderr)
            );
        }

        // Copy DLL to target directory
        let target_dll_path = target_dir.join("go_lib.dll");
        fs::copy(&dll_file, target_dll_path).expect("Failed to copy DLL to target directory");
    }

    // Linking settings
    println!("cargo:rustc-link-search=native={}", go_lib_dir);
    println!("cargo:rustc-link-lib=dylib=go_lib");

    // Use the rust version from Cargo.toml
    let rust_version = get_rust_version();

    // Generate Rust bindings
    let bindings = bindgen::Builder::default()
        .rust_target(rust_version.parse().unwrap())
        .header(&header_file)
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(bindings_path)
        .expect("Couldn't write bindings!");

    // Watch for changes
    println!("cargo:rerun-if-changed={}/go_lib.h", go_lib_dir);
    println!("cargo:rerun-if-changed={}/go_lib.go", go_lib_dir);
}

/// Returns the path to the `target/debug` directory, which is the directory where
/// the generated library should be copied to.
///
/// This function uses `cargo metadata` to get the target directory, and then
/// appends `debug` to it.
fn get_target_directory() -> PathBuf {
    // First try using CARGO_TARGET_DIR environment variable
    if let Ok(target_dir) = env::var("CARGO_TARGET_DIR") {
        return PathBuf::from(target_dir).join("debug");
    }

    // Fallback to cargo metadata
    let output = Command::new("cargo")
        .args(["metadata", "--format-version=1", "--no-deps"])
        .output()
        .expect("Failed to execute cargo metadata");

    if !output.status.success() {
        panic!(
            "cargo metadata command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("Failed to parse cargo metadata: {}", e));

    let target_directory = metadata["target_directory"]
        .as_str()
        .unwrap_or_else(|| panic!("Failed to get target directory from metadata"));

    // Get the profile (debug/release) from environment or default to debug
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    PathBuf::from(target_directory).join(profile)
}

fn get_rust_version() -> String {
    // Read Cargo.toml
    let cargo_toml = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");

    // Parse TOML
    let parsed_toml: toml::Value = toml::from_str(&cargo_toml).expect("Failed to parse Cargo.toml");

    // Get rust-version, default to "1.70" if not specified
    parsed_toml
        .get("package")
        .and_then(|package| package.get("rust-version"))
        .and_then(|version| version.as_str())
        .unwrap_or("1.70")
        .to_string()
}
