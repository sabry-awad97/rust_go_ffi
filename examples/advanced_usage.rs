use log::{debug, info, LevelFilter};
use rust_go_ffi::{self, add_numbers, cleanup, initialize};
use semver::Version;
use std::error::Error;

#[cfg(feature = "metrics")]
use metrics::{counter, histogram};

fn setup_logging() {
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Debug)
        .init();
}

fn run_ffi_operations() -> Result<(), Box<dyn Error>> {
    // Initialize with correct version (0.1.0)
    initialize(Version::new(0, 1, 0))?;

    // Print version information
    let version = rust_go_ffi::get_version()?;
    info!("DLL Version: {}", version);

    // Perform operations
    for i in 0..5 {
        let start = std::time::Instant::now();
        let result = add_numbers(i, i * 2)?;
        let duration = start.elapsed();

        info!("Operation result: {}", result);
        debug!("Operation took {:?}", duration);

        #[cfg(feature = "metrics")]
        {
            counter!("ffi.operations").increment(1);
            histogram!("ffi.operation.duration").record(duration);
        }
    }

    cleanup()?;
    Ok(())
}

#[cfg(feature = "metrics")]
fn print_metrics() {
    // Wait a moment for metrics to be collected
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Prometheus metrics are exposed via HTTP endpoint
    println!("\nMetrics are available at: http://localhost:9000/metrics");
    println!("Use curl http://localhost:9000/metrics to view them");

    // Keep the program running for a moment so metrics can be accessed
    std::thread::sleep(std::time::Duration::from_secs(5));
}

fn main() -> Result<(), Box<dyn Error>> {
    setup_logging();
    info!("Starting advanced FFI example");

    if let Err(e) = run_ffi_operations() {
        eprintln!("Error during FFI operations: {}", e);
        std::process::exit(1);
    }

    info!("FFI operations completed successfully");

    #[cfg(feature = "metrics")]
    print_metrics();

    Ok(())
}
