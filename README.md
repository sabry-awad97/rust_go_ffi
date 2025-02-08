# rust_go_ffi

## Overview

`rust_go_ffi` is a project that demonstrates how to create a Foreign Function Interface (FFI) between Rust and Go. This project includes a Rust library that calls functions defined in a Go library, showcasing interoperability between these two languages.

## Project Structure

- **src/**: Contains the Rust source code.
  - `main.rs`: The main entry point of the Rust application.
- **go_lib/**: Contains the Go source code.
  - `go_lib.go`: The Go library with exported functions.
  - `go.mod`: The Go module definition.
- **build.rs**: The build script for the Rust project, responsible for generating bindings and handling platform-specific tasks.
- **Cargo.toml**: The Rust project configuration file.

## Prerequisites

- Rust (version 1.70 or later)
- Go (version 1.23.4 or later)
- `bindgen` crate for generating Rust bindings from C headers
- `serde`, `serde_json`, and `toml` crates for configuration and metadata parsing

## Building the Project

1. **Clone the repository:**

     ```sh
     git clone https://github.com/sabry-awad97/rust_go_ffi.git
     cd rust_go_ffi
     ```

2. **Build the project:**

     ```sh
     cargo build
     ```

3. **Run the project:**

     ```sh
     cargo run
     ```

## Usage

The Rust application calls two functions from the Go library:

- `GoFunction`: Prints a message from Go.
- `AddNumbers`: Adds two integers and returns the result.

Example output:

```plaintext
Calling Go function from Rust...
Hello from Go!
5 + 10 = 15
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any improvements or bug fixes.

## Acknowledgements

- [bindgen](https://github.com/rust-lang/rust-bindgen) for generating Rust bindings.
- The Rust and Go communities for their excellent documentation and support.

## Contact

For any questions or inquiries, please contact [dr.sabry1997@gmail.com](mailto:dr.sabry1997@gmail.com).
