# Rust-Go FFI Bindings Generator

A professional-grade Foreign Function Interface (FFI) between Rust and Go, with automatic DLL management, version control, and advanced safety features.

## 🌟 Features

- **Safety First**
  - Safe Rust wrappers around unsafe Go FFI functions
  - Automatic DLL lifecycle management
  - Comprehensive error handling
  - Version compatibility checking

- **Development Features**
  - Colored console output for better UX
  - Detailed logging and debugging
  - Performance metrics tracking
  - Benchmarking tools

- **Production Ready**
  - Thread-safe DLL handling
  - Resource cleanup
  - Cross-platform support (Windows-focused)
  - Build-time binding generation

## 🔧 Prerequisites

- Rust 1.70+
- Go 1.16+
- Windows build tools:
  - MSVC toolchain
  - `dumpbin.exe`
  - `dlltool.exe`

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rust_go_ffi = { version = "0.1.0", features = ["auto-install"] }
```

Available features:

- `auto-install`: Enables automatic DLL installation
- `metrics`: Enables performance metrics
- `tracing`: Enables OpenTelemetry tracing
- `auto-cleanup`: Enables automatic resource cleanup

## 🚀 Usage

### Basic Usage

```rust
use rust_go_ffi::{add_numbers, initialize};
use semver::Version;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize with version check
    initialize(Version::new(0, 1, 0))?;

    // Use safe wrapper functions
    let result = add_numbers(5, 3)?;
    println!("5 + 3 = {}", result);

    Ok(())
}
```

### Advanced Usage

```rust
use rust_go_ffi::{self, add_numbers, get_version};
use log::{info, debug};

fn main() {
    // Setup logging
    env_logger::init();

    // Check DLL version
    let version = get_version().expect("Failed to get DLL version");
    info!("Using DLL version: {}", version);

    // Use with metrics
    #[cfg(feature = "metrics")]
    {
        let start = std::time::Instant::now();
        let result = add_numbers(10, 20).unwrap();
        metrics::histogram!("ffi.operation.duration").record(start.elapsed());
        debug!("Operation result: {}", result);
    }
}
```

## 🏗️ Building from Source

1. Clone and build:

```bash
git clone https://github.com/sabry-awad97/rust_go_ffi.git
cd rust_go_ffi
cargo build --features "auto-install metrics"
```

1. Run tests:

```bash
cargo test
RUST_LOG=debug cargo test -- --nocapture
cargo bench
```

## 📚 API Reference

### Core Functions

- `initialize(version: Version) -> Result<(), DllError>`
  - Initialize FFI system with version check
  
- `add_numbers(a: i32, b: i32) -> Result<i32, DllError>`
  - Safe wrapper for Go's addition function
  
- `get_version() -> Result<Version, DllError>`
  - Get current DLL version

### DLL Management

- `is_dll_available() -> bool`
  - Check DLL presence
  
- `verify_dll() -> Result<(), DllError>`
  - Verify DLL can be loaded
  
- `cleanup() -> Result<(), DllError>`
  - Clean up FFI resources

## 🔍 Troubleshooting

### Common Issues

1. **Version Mismatch**

   ```plaintext
   Error: VersionMismatch { expected: 1.0.0, found: 0.1.0 }
   ```

   - Ensure you're initializing with the correct version
   - Check Go library version

2. **DLL Loading Failed**

   ```plaintext
   Error: LoadError("Failed to load DLL")
   ```

   - Verify DLL is in PATH
   - Check Windows build tools

## 📊 Metrics

Enable metrics with the `metrics` feature:

```toml
[dependencies]
rust_go_ffi = { version = "0.1.0", features = ["metrics"] }
```

Available metrics:

- `ffi.calls`: Counter for FFI calls
- `ffi.errors`: Counter for errors
- `ffi.latency`: Histogram for call latency

## 🤝 Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

## 📜 License

Licensed under MIT - see [LICENSE](LICENSE)

## 📬 Contact

- Issues: [GitHub Issues](https://github.com/sabry-awad97/rust_go_ffi/issues)
- Email: [dr.sabry1997@gmail.com](mailto:dr.sabry1997@gmail.com)

---
Built with 💻 using Rust and Go
