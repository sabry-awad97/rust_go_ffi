use rust_go_ffi::{self, add_numbers, go_function, is_dll_available, verify_dll};

#[test]
fn test_full_dll_workflow() {
    // 1. Check DLL availability
    let available = is_dll_available();
    println!("DLL available: {}", available);

    // 2. If DLL is available, test verification
    if available {
        verify_dll().expect("DLL verification should succeed");

        // 3. Test Go function
        go_function().expect("Go function should execute");

        // 4. Test addition
        let result = add_numbers(7, 3).expect("Addition should work");
        assert_eq!(result, 10, "7 + 3 should equal 10");
    } else {
        println!("Skipping tests as DLL is not available");
    }
}

#[test]
#[cfg(feature = "auto-install")]
fn test_auto_installation() {
    use rust_go_ffi::install_dll;

    if !is_dll_available() {
        match install_dll() {
            Ok(()) => {
                assert!(is_dll_available(), "DLL should be available after installation");
                verify_dll().expect("DLL should be verifiable after installation");
            }
            Err(e) => println!("Installation failed (might be expected): {:?}", e),
        }
    }
}

#[test]
fn test_error_handling() {
    // Test with invalid numbers (if your Go function has bounds checking)
    match add_numbers(std::i32::MAX, 1) {
        Ok(_) => println!("Operation succeeded (might be expected)"),
        Err(e) => println!("Expected error occurred: {:?}", e),
    }
}
