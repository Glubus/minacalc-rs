use minacalc_rs::{Calc, HashMapCalcExt, RoxCalcExt};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MinaCalc ROX (Universal Chart Format) Example ===\n");

    let calc = Calc::new()?;
    println!("✅ Calculator created (version: {})\n", Calc::version());

    // Example 1: Load osu!mania chart (auto-detected format)
    let chart_path = PathBuf::from("assets/test.osu");

    if chart_path.exists() {
        println!("📁 Processing chart: {}", chart_path.display());

        // Calculate at 1.0x rate (default)
        println!("\n--- Calculating at 1.0x rate ---");
        let msd_1x = calc.calculate_msd_from_file(&chart_path, None)?;
        println!("✅ Chart processed successfully!");

        let hashmap_1x = msd_1x.as_hashmap()?;
        if let Some(scores) = hashmap_1x.get("1.0") {
            println!(
                "1.0x: Overall={:.2}, Stream={:.2}, Tech={:.2}",
                scores.overall, scores.stream, scores.technical
            );
        }

        // Calculate at 1.5x rate
        println!("\n--- Calculating at 1.5x rate ---");
        let msd_1_5x = calc.calculate_msd_from_file(&chart_path, Some(1.5))?;
        let hashmap_1_5x = msd_1_5x.as_hashmap()?;

        if let Some(scores) = hashmap_1_5x.get("1.0") {
            println!(
                "1.5x chart @ 1.0x calc: Overall={:.2}, Stream={:.2}, Tech={:.2}",
                scores.overall, scores.stream, scores.technical
            );
        }
        if let Some(scores) = hashmap_1_5x.get("1.5") {
            println!(
                "1.5x chart @ 1.5x calc: Overall={:.2}, Stream={:.2}, Tech={:.2}",
                scores.overall, scores.stream, scores.technical
            );
        }

        // Show some rates from 1.0x calculation
        println!("\n--- Available Rates (1.0x chart) ---");
        for (rate, scores) in hashmap_1x.iter().take(5) {
            println!(
                "{}: Overall={:.2}, Stream={:.2}",
                rate, scores.overall, scores.stream
            );
        }
    } else {
        println!("⚠️  Chart file not found: {}", chart_path.display());
        println!("   Place a .osu, .sm, or .rox file in assets/test.osu to test");
        println!("\n   ROX supports multiple formats:");
        println!("   - osu!mania (.osu)");
        println!("   - StepMania (.sm, .ssc)");
        println!("   - ROX binary (.rox)");
        println!("   - And more!");
    }

    println!("\n🎉 Example completed!");
    Ok(())
}
