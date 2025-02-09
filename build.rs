use std::env;
use std::path::PathBuf;
use std::process::Command;

const LIBRARY_PATH: &str = "go_lib";
const INPUT_HEADER: &str = "go_lib/go_lib.h";

fn main() {
    // Instruct Cargo when to re-run this build script.
    println!("cargo:rerun-if-changed=build.py");
    println!("cargo:rerun-if-changed=go_lib/go_lib.go");
    println!("cargo:rerun-if-changed=go_lib/go_lib.h");
    println!("cargo:rerun-if-changed=build.rs");

    // Execute the Python build script.
    // Adjust "python" to "python3" if needed.
    let status = Command::new("python")
        .arg("build.py")
        .status()
        .expect("Failed to execute build.py");
    if !status.success() {
        panic!(
            "build.py failed with exit status: {}",
            status.code().unwrap_or(-1)
        );
    }

    // Link configuration: Tell Cargo where to find the native library.
    println!("cargo:rustc-link-search=native={}", LIBRARY_PATH);
    // The library name here should match the actual library name without any prefix or extension.
    // For example, if your DLL is named "go_lib.dll", then use "go_lib".
    println!("cargo:rustc-link-lib=dylib=go_lib");

    // Generate Rust bindings to the provided header using bindgen.
    let bindings = bindgen::Builder::default()
        .rust_target("1.81".parse().unwrap())
        .header(INPUT_HEADER)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
