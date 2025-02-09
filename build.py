#!/usr/bin/env python3
"""
build.py

A professional build script that automates:
  - Building a Go shared library.
  - Generating a DEF file from the DLL exports.
  - Generating an import LIB file via dlltool.
  - Copying the DLL to the main target directory as well as test directories.
  - Building a Rust project with Cargo.
  - Optionally cleaning generated artifacts.

Usage:
    Build all targets:
        python build.py
    Clean build artifacts:
        python build.py --clean
"""

import argparse
import logging
import shutil
import subprocess
import sys
import time
from pathlib import Path

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="[%(asctime)s] %(levelname)s: %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S",
)

# Constants and Paths
FFI_DIR = Path("go_lib")
EXPORT_NAME = "go_lib"
EXPORT_GO = FFI_DIR / f"{EXPORT_NAME}.go"
EXPORT_DLL = FFI_DIR / f"{EXPORT_NAME}.dll"
EXPORT_DEF = FFI_DIR / f"{EXPORT_NAME}.def"
EXPORT_LIB = FFI_DIR / f"{EXPORT_NAME}.lib"
EXPORT_HEADER = FFI_DIR / f"{EXPORT_NAME}.h"

TARGET_DIR = Path("target") / "debug"
TARGET_DLL = TARGET_DIR / f"{EXPORT_NAME}.dll"

# Test directories where the DLL should also be copied.
TEST_DIRS = [Path("target/debug/deps"), Path("target/debug")]


def run_command(cmd, *, cwd=None, capture_output=False, shell=False):
    """
    Helper to run a command and return the result.
    Raises CalledProcessError if the command fails.
    """
    logging.info("Running command: %s", " ".join(cmd) if isinstance(cmd, list) else cmd)
    result = subprocess.run(
        cmd, cwd=cwd, capture_output=capture_output, text=True, shell=shell, check=True
    )
    return result


def ensure_dirs():
    """Ensure the FFI directory exists."""
    if not FFI_DIR.exists():
        logging.info("Creating directory: %s", FFI_DIR)
        FFI_DIR.mkdir(parents=True, exist_ok=True)
    else:
        logging.info("Directory already exists: %s", FFI_DIR)


def go_mod_init():
    """
    Initialize the Go module in the FFI directory.
    If go.mod already exists, log that the module is already initialized.
    """
    go_mod = FFI_DIR / "go.mod"
    if go_mod.exists():
        logging.info("Go module already initialized (%s exists)", go_mod)
        return

    try:
        logging.info("Initializing Go module...")
        run_command(["go", "mod", "init", "whatsmeow-ffi"], cwd=str(FFI_DIR))
    except subprocess.CalledProcessError as e:
        logging.error("Failed to initialize Go module: %s", e)
        sys.exit(1)


def go_build():
    """
    Build the Go shared library.
    This command is executed from within FFI_DIR so that the output DLL
    does not include extra path components.
    """
    logging.info("Building Go shared library...")
    try:
        # Run in FFI_DIR so that output file is just "go_lib.dll"
        run_command(
            [
                "go",
                "build",
                "-buildmode=c-shared",
                "-o",
                f"{EXPORT_NAME}.dll",
                f"{EXPORT_NAME}.go",
            ],
            cwd=str(FFI_DIR),
        )
    except subprocess.CalledProcessError as e:
        logging.error("Go build failed: %s", e)
        sys.exit(1)

    if not EXPORT_DLL.exists():
        logging.error("Expected DLL not found: %s", EXPORT_DLL)
        sys.exit(1)
    else:
        logging.info("DLL built: %s", EXPORT_DLL)


def generate_def_content(dumpbin_output: str) -> str:
    """
    Process dumpbin output to generate DEF file content.
    Lines that start with a digit and have at least 4 columns are parsed,
    and the 4th token (the symbol name) is extracted.
    """
    lines = dumpbin_output.splitlines()
    def_lines = ["EXPORTS"]
    for line in lines:
        parts = line.split()
        if len(parts) > 3 and parts[0].isdigit():
            def_lines.append(parts[3])
    return "\n".join(def_lines)


def generate_def():
    """
    Generate a DEF file from the DLL exports using dumpbin.
    Note: Since we're running in FFI_DIR, we pass only the DLL's name.
    """
    logging.info("Generating DEF file...")
    try:
        # Use EXPORT_DLL.name so that we pass "go_lib.dll" instead of the full path.
        result = run_command(
            ["dumpbin", "/exports", EXPORT_DLL.name],
            cwd=str(FFI_DIR),
            capture_output=True,
        )
    except subprocess.CalledProcessError as e:
        logging.error("dumpbin failed: %s", e)
        sys.exit(1)

    def_content = generate_def_content(result.stdout)
    try:
        with EXPORT_DEF.open("w", encoding="utf-8") as f:
            f.write(def_content)
        logging.info("DEF file generated: %s", EXPORT_DEF)
    except IOError as e:
        logging.error("Failed to write DEF file: %s", e)
        sys.exit(1)


def generate_lib():
    """
    Generate an import library (.lib) from the DEF file using dlltool.
    To avoid path component issues, this command is executed from FFI_DIR.
    """
    logging.info("Generating import library (.lib) using dlltool...")
    try:
        run_command(
            [
                "dlltool",
                "-d",
                EXPORT_DEF.name,
                "-D",
                EXPORT_DLL.name,
                "-l",
                EXPORT_LIB.name,
            ],
            cwd=str(FFI_DIR),
        )
    except subprocess.CalledProcessError as e:
        logging.error("dlltool failed: %s", e)
        sys.exit(1)
    logging.info("Import library generated: %s", EXPORT_LIB)


def copy_with_retry(src: Path, dst: Path, retries: int) -> None:
    """
    Attempt to copy a file from src to dst with a simple retry mechanism.
    """
    for attempt in range(1, retries + 1):
        try:
            shutil.copy2(str(src), str(dst))
            return
        except Exception as e:
            if attempt < retries:
                logging.warning(
                    "Failed to copy %s to %s (attempt %d/%d). Retrying...",
                    src,
                    dst,
                    attempt,
                    retries,
                )
                time.sleep(0.1 * attempt)
            else:
                logging.error(
                    "Failed to copy %s to %s after %d attempts: %s",
                    src,
                    dst,
                    retries,
                    e,
                )
                raise e


def copy_dll():
    """
    Copy the generated DLL to the primary target directory.
    """
    logging.info("Copying DLL to target directory: %s", TARGET_DIR)
    TARGET_DIR.mkdir(parents=True, exist_ok=True)
    try:
        shutil.copy2(str(EXPORT_DLL), str(TARGET_DLL))
    except Exception as e:
        logging.error("Failed to copy DLL: %s", e)
        sys.exit(1)
    logging.info("DLL copied to: %s", TARGET_DLL)


def copy_dll_to_test_dir():
    """
    Copy the DLL to additional test directories.
    This mimics the Rust function that copies the DLL to both
    'target/debug/deps' and 'target/debug'.
    """
    for test_dir in TEST_DIRS:
        test_dir.mkdir(parents=True, exist_ok=True)
        target_dll = test_dir / f"{EXPORT_NAME}.dll"
        try:
            copy_with_retry(EXPORT_DLL, target_dll, retries=5)
        except Exception as e:
            logging.error("Failed to copy DLL to test directory %s: %s", test_dir, e)
            sys.exit(1)
        logging.info("Copied DLL to test directory: %s", target_dll)


def cargo_build():
    """
    Build the Rust project with Cargo.
    """
    logging.info("Building Rust project with Cargo...")
    try:
        run_command(["cargo", "build"], capture_output=False)
    except subprocess.CalledProcessError as e:
        logging.error("Cargo build failed: %s", e)
        sys.exit(1)
    logging.info("Rust project built successfully.")


def clean():
    """
    Remove generated files and run cargo clean.
    """
    logging.info("Cleaning up generated files...")
    for path in [EXPORT_DLL, EXPORT_DEF, EXPORT_LIB]:
        if path.exists():
            try:
                path.unlink()
                logging.info("Deleted: %s", path)
            except Exception as e:
                logging.warning("Could not delete %s: %s", path, e)
    logging.info("Running 'cargo clean'...")
    try:
        run_command(["cargo", "clean"])
    except subprocess.CalledProcessError as e:
        logging.error("cargo clean failed: %s", e)
        sys.exit(1)


def build_all():
    """
    Execute the entire build process.
    """
    ensure_dirs()
    go_mod_init()
    go_build()
    generate_def()
    generate_lib()
    copy_dll()
    copy_dll_to_test_dir()  # New: Copy DLL to test directories.
    # cargo_build()


def parse_args():
    """
    Parse command-line arguments.
    """
    parser = argparse.ArgumentParser(
        description="Build Go shared library and Rust project."
    )
    parser.add_argument(
        "--clean",
        action="store_true",
        help="Clean generated artifacts and cargo build artifacts.",
    )
    return parser.parse_args()


def main():
    args = parse_args()
    if args.clean:
        clean()
    else:
        build_all()


if __name__ == "__main__":
    main()
