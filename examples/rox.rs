use minacalc_rs::{Calc, HashMapCalcExt, RoxCalcExt};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MinaCalc ROX (Universal Chart Format) Example ===\n");

    let calc = Calc::new()?;
    println!("✅ Calculator created (version: {})\n", Calc::version());

    // Test 4K, 6K, and 7K charts
    let charts = [
        ("assets/4K.osu", "4K"),
        ("assets/6K.osu", "6K"),
        ("assets/7K.osu", "7K"),
    ];

    for (path, name) in charts {
        let chart_path = PathBuf::from(path);

        if chart_path.exists() {
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("📁 Processing {} chart: {}", name, chart_path.display());
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

            // Single Rate SSR Calculation
            println!("--- Single Rate SSR (1.0x @ 93%) ---");
            // Direct call, propagating error if any
            let ssr = calc.calculate_ssr_from_file(&chart_path, 1.0, 93.0, None)?;

            println!("✅ SSR calculated!");
            println!(
                "   Overall: {:.2}, Stream: {:.2}, Tech: {:.2}",
                ssr.overall, ssr.stream, ssr.technical
            );
            println!(
                "   Jumpstream: {:.2}, Handstream: {:.2}, Stamina: {:.2}",
                ssr.jumpstream, ssr.handstream, ssr.stamina
            );
            println!(
                "   -> (Aliases: Chordstream: {:.2}, Bracketing: {:.2})",
                ssr.chordstream(),
                ssr.bracketing()
            );
            println!(
                "   Jackspeed: {:.2}, Chordjack: {:.2}",
                ssr.jackspeed, ssr.chordjack
            );

            // All rates calculation
            // Calculate all rates MSD (MinaSD)
            println!("\n--- All Rates MSD ---");
            let msd = calc.calculate_all_rates_from_file(&chart_path)?;
            println!("✅ All rates calculated!");

            let hashmap = msd.as_hashmap()?;
            println!("\n   Sample rates:");
            for rate in ["0.7", "1.0", "1.5", "2.0"] {
                if let Some(scores) = hashmap.get(rate) {
                    println!(
                        "   {}x: Overall={:.2}, Stream={:.2}, Tech={:.2}",
                        rate, scores.overall, scores.stream, scores.technical
                    );
                }
            }

            // Calculate with chart rate (1.5x chart speed)
            println!("\n--- Chart at 1.5x Speed ---");
            let ssr_1_5x = calc.calculate_ssr_from_file(&chart_path, 1.0, 93.0, Some(1.5))?;
            println!("   1.5x chart @ 1.0x calc @ 93%:");
            println!(
                "   Overall: {:.2}, Stream: {:.2}, Tech: {:.2}\n",
                ssr_1_5x.overall, ssr_1_5x.stream, ssr_1_5x.technical
            );
        } else {
            println!("⚠️  {} chart not found: {}", name, chart_path.display());
        }
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🎉 Example completed!");
    println!("\nROX supports parsing multiple formats:");
    println!("   - osu!mania (.osu)");
    println!("   - StepMania (.sm, .ssc)");
    println!("   - ROX binary (.rox)");
    println!("   - And more!");
    Ok(())
}
