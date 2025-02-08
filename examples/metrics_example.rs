use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use colored::*;
use metrics::{counter, describe_counter, describe_gauge, describe_histogram, gauge, histogram};
use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_util::MetricKindMask;
use rust_go_ffi::{add_numbers, initialize, verify_dll};
use semver::Version;

fn setup_metrics() {
    let builder = PrometheusBuilder::new();
    builder
        .idle_timeout(
            MetricKindMask::COUNTER | MetricKindMask::HISTOGRAM,
            Some(Duration::from_secs(10)),
        )
        .with_http_listener(([127, 0, 0, 1], 9000))
        .install()
        .expect("failed to install Prometheus recorder");

    // Register metrics with descriptions
    describe_counter!("ffi_calls_total", "Total number of FFI function calls made");
    describe_histogram!(
        "ffi_call_duration_ms",
        "Duration of FFI calls in milliseconds"
    );
    describe_gauge!(
        "ffi_operations_success_rate",
        "Success rate of FFI operations"
    );
}

fn run_ffi_operations() {
    let start = std::time::Instant::now();

    // Initialize FFI
    if let Err(e) = initialize(Version::new(0, 1, 0)) {
        eprintln!("Failed to initialize FFI: {:?}", e);
        return;
    }

    // Verify DLL
    if let Err(e) = verify_dll() {
        eprintln!("Failed to verify DLL: {:?}", e);
        return;
    }

    let mut success_count = 0;
    let total_operations = 10;

    println!("{}", "\nRunning FFI operations...".cyan().bold());
    for i in 0..total_operations {
        print!("Operation {}/{}: ", i + 1, total_operations);

        let op_start = std::time::Instant::now();
        match add_numbers(i as i32, (i * 2) as i32) {
            Ok(result) => {
                success_count += 1;
                counter!("ffi_calls_total", "operation" => "add_numbers").increment(1);
                histogram!("ffi_call_duration_ms").record(op_start.elapsed().as_millis() as f64);
                println!("{} ({})", "SUCCESS".green().bold(), result);
            }
            Err(e) => {
                counter!(
                    "ffi_calls_total",
                    "operation" => "add_numbers",
                    "status" => "error"
                )
                .increment(1);
                println!("{} ({:?})", "FAILED".red().bold(), e);
            }
        }

        // Update success rate gauge
        let success_rate = (success_count as f64 / (i + 1) as f64) * 100.0;
        gauge!("ffi_operations_success_rate").set(success_rate);

        thread::sleep(Duration::from_millis(500));
    }

    println!(
        "\n{} Operations completed in {:?}",
        "✓".green().bold(),
        start.elapsed()
    );
    println!(
        "{} Success rate: {:.1}%",
        "📊".cyan(),
        (success_count as f64 / total_operations as f64) * 100.0
    );
}

fn cleanup_and_exit() {
    println!("\n{}", "Shutting down gracefully...".yellow());
    // Add small delay for any pending operations
    thread::sleep(Duration::from_millis(200));
    println!("{}", "Goodbye! 👋".green().bold());
    // Ensure we exit with success status
    std::process::exit(0);
}

fn main() {
    // Windows-specific: Set console control handler
    #[cfg(windows)]
    unsafe {
        use winapi::um::consoleapi::SetConsoleCtrlHandler;

        extern "system" fn handler(_: u32) -> i32 {
            cleanup_and_exit();
            0
        }

        SetConsoleCtrlHandler(Some(handler), 1);
    }

    println!("{}", "FFI Metrics Example".bold().blue());
    println!("{}", "=================".blue());

    // Set up Ctrl+C handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        cleanup_and_exit();
    })
    .expect("Error setting Ctrl-C handler");

    setup_metrics();
    println!(
        "{}",
        "Metrics server started at http://127.0.0.1:9000".yellow()
    );

    run_ffi_operations();

    println!("\n{}", "Keeping metrics server alive...".cyan());
    println!("Press Ctrl+C to exit");

    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_secs(1));
    }

    cleanup_and_exit();
}
