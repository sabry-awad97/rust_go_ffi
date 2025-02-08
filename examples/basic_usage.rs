use colored::*;
use rust_go_ffi::{self, is_dll_available, verify_dll};

fn main() {
    println!("{}", "Rust-Go FFI Example".bold().blue());
    println!("{}", "==================".blue());

    // Check DLL availability
    print!("{}", "🔍 Checking DLL availability... ".yellow());
    if !is_dll_available() {
        println!("{}", "NOT FOUND".red().bold());

        #[cfg(feature = "auto-install")]
        {
            println!("{}", "\n📥 Attempting automatic installation...".cyan());
            match rust_go_ffi::install_dll() {
                Ok(()) => println!("{}", "✅ DLL installed successfully".green()),
                Err(e) => {
                    eprintln!("{} {:?}", "❌ Installation failed:".red().bold(), e);
                    std::process::exit(1);
                }
            }
        }
        #[cfg(not(feature = "auto-install"))]
        {
            eprintln!("{}", "\n❌ Auto-install feature not enabled".red().bold());
            eprintln!(
                "{}",
                "Please enable the 'auto-install' feature or install manually".yellow()
            );
            std::process::exit(1);
        }
    } else {
        println!("{}", "FOUND".green().bold());
    }

    // Verify DLL
    print!("{}", "\n🔍 Verifying DLL... ".yellow());
    if let Err(e) = verify_dll() {
        println!("{}", "FAILED".red().bold());
        eprintln!("{} {:?}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
    println!("{}", "OK".green().bold());

    println!("\n{}", "📋 Testing FFI Functions:".cyan().bold());
    println!("{}", "=====================".cyan());

    unsafe {
        print!("Adding numbers (5 + 3)... ");
        let result = rust_go_ffi::AddNumbers(5, 3);
        println!("{} = {} {}", "result".blue().bold(), result, "✅".green());
    }

    println!(
        "\n{}",
        "✨ All tests completed successfully! ✨".green().bold()
    );
}
