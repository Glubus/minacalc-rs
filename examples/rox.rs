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

        // Example 1: Calculate SSR for a single rate (efficient!)
        println!("\n--- Single Rate SSR Calculation (1.0x @ 93%) ---");
        let ssr = calc.calculate_ssr_from_file(&chart_path, 1.0, 93.0, None)?;
        println!("✅ SSR calculated!");
        println!(
            "Overall: {:.2}, Stream: {:.2}, Tech: {:.2}",
            ssr.overall, ssr.stream, ssr.technical
        );

        // Example 2: Calculate all rates (0.7x to 2.0x)
        println!("\n--- All Rates MSD Calculation ---");
        let all_rates = calc.calculate_all_rates_from_file(&chart_path, None)?;
        println!("✅ All rates calculated!");

        let hashmap = all_rates.as_hashmap()?;
        println!("\nSample rates:");
        for rate in ["0.7", "1.0", "1.5", "2.0"] {
            if let Some(scores) = hashmap.get(rate) {
                println!(
                    "{}x: Overall={:.2}, Stream={:.2}, Tech={:.2}",
                    rate, scores.overall, scores.stream, scores.technical
                );
            }
        }

        // Example 3: Calculate with chart rate (1.5x chart speed)
        println!("\n--- Chart at 1.5x Speed ---");
        let ssr_1_5x = calc.calculate_ssr_from_file(&chart_path, 1.0, 93.0, Some(1.5))?;
        println!("1.5x chart @ 1.0x calc @ 93%:");
        println!(
            "Overall: {:.2}, Stream: {:.2}, Tech: {:.2}",
            ssr_1_5x.overall, ssr_1_5x.stream, ssr_1_5x.technical
        );
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
