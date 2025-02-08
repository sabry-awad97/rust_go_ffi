# Rust-Go FFI Bindings Generator

A professional-grade Foreign Function Interface (FFI) between Rust and Go, with automatic DLL management and colored console output.

## 🌟 Features

- Automatic DLL installation and management
- Safe Rust wrappers around Go functions
- Colored console output for better UX
- Comprehensive error handling
- Cross-platform support (Windows-focused)
- Build-time binding generation

## 🔧 Prerequisites

- Rust (1.70+)
- Go (1.16+)
- Windows build tools:
  - MSVC toolchain
  - `dumpbin.exe`
  - `dlltool.exe`

## 📦 Installation

1. Add to your `Cargo.toml`:

```toml
[dependencies]
rust_go_ffi = { version = "0.1.0", features = ["auto-install"] }
```

1. Import and use:

```rust
use rust_go_ffi::{add_numbers, go_function};

fn main() {
    // Safe, managed function calls
    if let Ok(sum) = add_numbers(5, 3) {
        println!("5 + 3 = {}", sum);
    }
}
```

## 🚀 Usage

### Basic Usage

```rust
use rust_go_ffi::{self, verify_dll, is_dll_available};

fn main() {
    // Verify DLL is available
    if !is_dll_available() {
        panic!("DLL not found!");
    }

    // Initialize FFI
    verify_dll().expect("Failed to load DLL");

    // Call Go functions
    let result = add_numbers(5, 10).expect("Failed to add numbers");
    println!("Result: {}", result);
}
```

### Features

- `auto-install`: Enables automatic DLL installation

  ```toml
  [dependencies]
  rust_go_ffi = { version = "0.1.0", features = ["auto-install"] }
  ```

## 🔨 Building from Source

1. Clone the repository:

```bash
git clone https://github.com/sabry-awad97/rust_go_ffi.git
cd rust_go_ffi
```

1. Build the project:

```bash
cargo build
```

1. Run tests:

```bash
cargo test
cargo run --example basic_usage
```

## 📚 API Reference

### Safe Functions

- `verify_dll() -> Result<(), DllError>`
  - Verifies DLL can be loaded
  
- `is_dll_available() -> bool`
  - Checks if DLL exists in system
  
- `add_numbers(a: i32, b: i32) -> Result<i32, DllError>`
  - Safe wrapper for Go's AddNumbers function
  
- `go_function() -> Result<(), DllError>`
  - Safe wrapper for Go's GoFunction

### Feature-gated Functions

With `auto-install` feature:

- `install_dll() -> Result<(), DllError>`
  - Automatically installs DLL to system

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔍 Troubleshooting

### Common Issues

1. **DLL Not Found**

   ```sh
   ⚠️ DLL not found in system
   ```

   - Enable auto-install feature
   - Check DLL path in PATH environment variable

2. **Access Violation**

   ```sh
   STATUS_ACCESS_VIOLATION
   ```

   - Verify DLL compatibility
   - Check function signatures match

## ✨ Credits

- Built with [bindgen](https://github.com/rust-lang/rust-bindgen)
- Console colors by [colored](https://github.com/mackwic/colored)

## 📬 Contact

For support or queries:

- Create an issue in the repository
- Email: <dr.sabry1997@gmail.com>

---
Built with ❤️ using Rust and Go
