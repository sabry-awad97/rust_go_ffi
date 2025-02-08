extern crate bindgen;

use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let go_lib_dir = "go_lib"; // Directory containing Go files
    let go_source = format!("{}/go_lib.go", go_lib_dir);
    let dll_file = format!("{}/go_lib.dll", go_lib_dir);

    let header_file = format!("{}/go_lib.h", go_lib_dir);
    let out_dir = env::var("OUT_DIR").unwrap();
    let bindings_path = Path::new(&out_dir).join("bindings.rs");
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    // Build the Go shared library
    let output = Command::new("go")
        .args([
            "build",
            "-buildmode=c-shared",
            "-o",
            dll_file.as_str(),
            go_source.as_str(),
        ])
        .output()
        .expect("Failed to execute go build");

    if !output.status.success() {
        panic!(
            "Failed to build Go code: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // If Windows, generate .lib and .def files
    if target_os == "windows" {
        let def_file = format!("{}/go_lib.def", go_lib_dir);
        let lib_file = format!("{}/go_lib.lib", go_lib_dir);

        // Generate .def file using dumpbin
        let dumpbin_output = Command::new("dumpbin")
            .args(["/exports", dll_file.as_str()])
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
        std::fs::write(def_file.as_str(), def_content).expect("Failed to write .def file");

        // Generate .lib using dlltool
        let dlltool_output = Command::new("dlltool")
            .args([
                "-d",
                def_file.as_str(),
                "-D",
                dll_file.as_str(),
                "-l",
                lib_file.as_str(),
            ])
            .output()
            .expect("Failed to run dlltool");

        if !dlltool_output.status.success() {
            panic!(
                "dlltool failed: {}",
                String::from_utf8_lossy(&dlltool_output.stderr)
            );
        }
    }

    // Linking settings
    println!("cargo:rustc-link-search=native=./go_lib");
    println!("cargo:rustc-link-lib=dylib=go_lib");

    // Generate Rust bindings
    let bindings = bindgen::Builder::default()
        .rust_target("1.81".parse().unwrap())
        .header(header_file)
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(bindings_path)
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=go_lib/go_lib.h");
}
