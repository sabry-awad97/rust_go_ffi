# Windows Makefile for Rust-Go FFI Project

# Configuration
SHELL := cmd.exe
.SHELLFLAGS := /c
CARGO := cargo
GO := go
RM := del /Q
RMDIR := rmdir /S /Q
MKDIR := mkdir
COPY := copy /Y

# Directories
GO_LIB_DIR := go_lib
TARGET_DIR := target
BENCH_DIR := $(TARGET_DIR)\criterion
DEBUG_DIR := $(TARGET_DIR)\debug
RELEASE_DIR := $(TARGET_DIR)\release

# Files
DLL_NAME := go_lib.dll
DLL_PATH := $(GO_LIB_DIR)\$(DLL_NAME)

# Rust Features
RUST_FEATURES := auto-install,metrics

# Default target
.PHONY: all
all: build-all test-all

# Build targets
.PHONY: build-all
build-all: build-go build-rust

.PHONY: build-go
build-go:
	@echo Building Go DLL...
	@if not exist $(GO_LIB_DIR) $(MKDIR) $(GO_LIB_DIR)
	@cd $(GO_LIB_DIR) && $(GO) build -buildmode=c-shared -o $(DLL_NAME) go_lib.go

.PHONY: build-rust
build-rust: build-go
	@echo Building Rust project...
	$(CARGO) build --features $(RUST_FEATURES)

.PHONY: build-release
build-release: build-go
	@echo Building Rust project (release)...
	$(CARGO) build --release --features $(RUST_FEATURES)

# Test targets
.PHONY: test-all
test-all: test-rust test-go

.PHONY: test-rust
test-rust:
	@echo Running Rust tests...
	$(CARGO) test --features $(RUST_FEATURES) -- --nocapture

.PHONY: test-go
test-go:
	@echo Running Go tests...
	@cd $(GO_LIB_DIR) && $(GO) test -v

# Test with all features
.PHONY: test-all-features
test-all-features:
	@echo Running tests with all features...
	$(CARGO) test --features "auto-install,metrics,tracing,auto-cleanup" --all-targets
	$(CARGO) run --example metrics_example --features "auto-install,metrics,tracing,auto-cleanup"
	$(CARGO) run --example advanced_usage --features "auto-install,metrics,tracing,auto-cleanup"

# Benchmark targets
.PHONY: bench
bench: build-release
	@echo Running benchmarks...
	$(CARGO) bench
	@echo Benchmark results available in $(BENCH_DIR)

# Documentation
.PHONY: doc
doc:
	@echo Generating documentation...
	$(CARGO) doc --no-deps --features $(RUST_FEATURES)
	@echo Documentation available in $(TARGET_DIR)\doc

# Clean targets
.PHONY: clean
clean: clean-rust clean-go

.PHONY: clean-rust
clean-rust:
	@echo Cleaning Rust build...
	$(CARGO) clean

.PHONY: clean-go
clean-go:
	@echo Cleaning Go build...
	@if exist $(GO_LIB_DIR)\$(DLL_NAME) $(RM) $(GO_LIB_DIR)\$(DLL_NAME)
	@if exist $(GO_LIB_DIR)\*.h $(RM) $(GO_LIB_DIR)\*.h

# Install/Uninstall targets
.PHONY: install
install: build-release
	@echo Installing...
	@if not exist "$(HOME)\.cargo\bin" $(MKDIR) "$(HOME)\.cargo\bin"
	$(COPY) "$(RELEASE_DIR)\$(DLL_NAME)" "$(HOME)\.cargo\bin"

.PHONY: uninstall
uninstall:
	@echo Uninstalling...
	@if exist "$(HOME)\.cargo\bin\$(DLL_NAME)" $(RM) "$(HOME)\.cargo\bin\$(DLL_NAME)"

# Development helpers
.PHONY: check
check:
	@echo Running cargo check...
	$(CARGO) check --features $(RUST_FEATURES)
	@echo Running clippy...
	$(CARGO) clippy --features $(RUST_FEATURES)
	@echo Running formatting check...
	$(CARGO) fmt -- --check

.PHONY: format
format:
	@echo Formatting code...
	$(CARGO) fmt

# Help target
.PHONY: help
help:
	@echo Available targets:
	@echo   all          - Build and test everything (default)
	@echo   build-all    - Build both Go and Rust components
	@echo   build-go     - Build only Go DLL
	@echo   build-rust   - Build Rust project (debug)
	@echo   build-release- Build Rust project (release)
	@echo   test-all     - Run all tests
	@echo   bench        - Run benchmarks
	@echo   doc          - Generate documentation
	@echo   clean        - Clean all build artifacts
	@echo   install      - Install to cargo bin
	@echo   uninstall    - Remove from cargo bin
	@echo   check        - Run all checks (clippy, fmt)
	@echo   format       - Format code
