//! Example: Multi-threaded chart calculation
//!
//! Demonstrates using ThreadCalc to calculate multiple charts concurrently.

use minacalc_rs::rox::calc::high_level::RoxCalcExt;
use minacalc_rs::thread::ThreadCalc;
use std::path::PathBuf;
use std::thread;
use std::time::Instant;

fn main() {
    // Find all chart files in assets directory
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    let chart_files: Vec<PathBuf> = std::fs::read_dir(&assets_dir)
        .expect("Failed to read assets directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .map(|ext| {
                    let ext = ext.to_string_lossy().to_lowercase();
                    ext == "osu" || ext == "sm" || ext == "ssc"
                })
                .unwrap_or(false)
        })
        .collect();

    println!(
        "Found {} chart files in {:?}",
        chart_files.len(),
        assets_dir
    );

    if chart_files.is_empty() {
        println!("No chart files found! Add .osu, .sm, or .ssc files to assets/");
        return;
    }

    // Calculate all charts in parallel using threads
    let start = Instant::now();

    let handles: Vec<_> = chart_files
        .into_iter()
        .map(|path| {
            thread::spawn(move || {
                // Each thread gets its own ThreadCalc (thread-local singleton)
                let calc = ThreadCalc::new().expect("Failed to create ThreadCalc");

                let result = calc.calculate_at_rate_from_file(
                    &path, 1.0,  // music rate
                    0.93, // score goal
                    None, // chart rate
                    true, // capped (SSR mode)
                );

                match result {
                    Ok(scores) => {
                        println!(
                            "[{:?}] {} -> Overall: {:.2}",
                            thread::current().id(),
                            path.file_name().unwrap().to_string_lossy(),
                            scores.overall
                        );
                        Some((path, scores))
                    }
                    Err(e) => {
                        eprintln!(
                            "[{:?}] {} -> Error: {}",
                            thread::current().id(),
                            path.file_name().unwrap().to_string_lossy(),
                            e
                        );
                        None
                    }
                }
            })
        })
        .collect();

    // Collect results
    let results: Vec<_> = handles
        .into_iter()
        .filter_map(|h| h.join().ok().flatten())
        .collect();

    let elapsed = start.elapsed();

    println!("\n=== Results ===");
    println!("Calculated {} charts in {:?}", results.len(), elapsed);

    for (path, scores) in &results {
        println!(
            "  {} -> O:{:.2} S:{:.2} JS:{:.2} HS:{:.2} CJ:{:.2} T:{:.2}",
            path.file_name().unwrap().to_string_lossy(),
            scores.overall,
            scores.stream,
            scores.jumpstream,
            scores.handstream,
            scores.chordjack,
            scores.technical
        );
    }
}
