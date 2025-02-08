use colored::*;
use rust_go_ffi::{self, add_numbers, go_function, is_dll_available, verify_dll};

fn main() {
    println!("{}", "Rust-Go FFI Interface".bold().green());
    println!("{}", "===================".green());

    // Check DLL availability
    if !is_dll_available() {
        eprintln!("{}", "⚠️ DLL not found in system".yellow().bold());
        std::process::exit(1);
    }

    // Verify DLL can be loaded
    print!("{}", "🔍 Verifying DLL... ".blue());
    match verify_dll() {
        Ok(_) => println!("{}", "✅ OK".green()),
        Err(e) => {
            println!("{}", "❌ FAILED".red());
            eprintln!("{} {:?}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }

    println!("\n{}", "📋 Running FFI Tests:".cyan().bold());
    println!("{}", "-------------------".cyan());

    // Test Go function
    print!("Testing Go function... ");
    match go_function() {
        Ok(_) => println!("{}", "✅".green()),
        Err(e) => {
            println!("{}", "❌ FAILED".red());
            eprintln!("{} {:?}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }

    // Test addition
    print!("Testing addition: 5 + 10 = ");
    match add_numbers(5, 10) {
        Ok(result) => println!("{}{}", result, " ✅".green()),
        Err(e) => {
            println!("{}", "❌ FAILED".red());
            eprintln!("{} {:?}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }

    println!(
        "\n{}",
        "✨ All tests completed successfully! ✨".green().bold()
    );
}
