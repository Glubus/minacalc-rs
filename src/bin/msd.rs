//! MinaCalc MSD Calculator CLI
//!
//! Simple and powerful CLI for calculating rhythm game difficulty ratings.
//!
//! Usage:
//!   msd <file>              - Calculate MSD for all rates (0.7x - 2.0x)
//!   msd <file> --rate 1.0   - Calculate SSR at specific rate
//!   msd <file> --json       - Output as JSON

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use minacalc_rs::{Calc, RoxCalcExt, SkillsetScores};

fn main() -> ExitCode {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        print_usage();
        return ExitCode::SUCCESS;
    }

    let file_path = PathBuf::from(&args[1]);
    let json_output = args.iter().any(|a| a == "--json" || a == "-j");
    let capped = args.iter().any(|a| a == "--capped");
    let rate = parse_rate(&args);

    match run(&file_path, rate, json_output, capped) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn print_usage() {
    eprintln!(
        r#"MinaCalc MSD Calculator v{}

USAGE:
    msd <file> [OPTIONS]

OPTIONS:
    -r, --rate <RATE>   Calculate Difficulty at specific rate (default: all rates)
    --capped            Use Capped (SSR) calculation (default: Uncapped/MSD)
    -j, --json          Output as JSON
    -h, --help          Show this help

EXAMPLES:
    msd chart.osu                    # All rates, Uncapped (MSD)
    msd chart.osu --capped           # All rates, Capped (SSR)
    msd chart.osu --rate 1.0         # Single rate 1.0x, MSD
    msd chart.osu -r 1.0 --capped    # Single rate 1.0x, SSR
"#,
        env!("CARGO_PKG_VERSION")
    );
}

fn parse_rate(args: &[String]) -> Option<f32> {
    for (i, arg) in args.iter().enumerate() {
        if (arg == "--rate" || arg == "-r") && i + 1 < args.len() {
            return args[i + 1].parse().ok();
        }
    }
    None
}

fn run(
    path: &Path,
    rate: Option<f32>,
    json: bool,
    capped: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()).into());
    }

    let calc = Calc::new()?;

    match rate {
        Some(r) => output_single_rate(&calc, path, r, json, capped),
        None => output_all_rates(&calc, path, json, capped),
    }
}

fn output_single_rate(
    calc: &Calc,
    path: &Path,
    rate: f32,
    json: bool,
    capped: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // defaults: score_goal=0.93 (ignored if !capped), chart_rate=None
    let scores = calc.calculate_at_rate_from_file(path, rate, 0.93, None, capped)?;

    if json {
        println!(
            r#"{{"rate":{},"capped":{},"overall":{:.2},"stream":{:.2},"jumpstream":{:.2},"handstream":{:.2},"stamina":{:.2},"jackspeed":{:.2},"chordjack":{:.2},"technical":{:.2}}}"#,
            rate,
            capped,
            scores.overall,
            scores.stream,
            scores.jumpstream,
            scores.handstream,
            scores.stamina,
            scores.jackspeed,
            scores.chordjack,
            scores.technical
        );
    } else {
        print_scores_human(&scores, Some(rate), capped);
    }

    Ok(())
}

fn output_all_rates(
    calc: &Calc,
    path: &Path,
    json: bool,
    capped: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let all_rates = calc.calculate_all_rates_from_file(path, capped)?;

    if json {
        print!("[");
        for (i, (rate, scores)) in RATES.iter().zip(all_rates.msds.iter()).enumerate() {
            if i > 0 {
                print!(",");
            }
            println!(
                r#"{{"rate":{},"capped":{},"overall":{:.2},"stream":{:.2},"jumpstream":{:.2},"handstream":{:.2},"stamina":{:.2},"jackspeed":{:.2},"chordjack":{:.2},"technical":{:.2}}}"#,
                rate,
                capped,
                scores.overall,
                scores.stream,
                scores.jumpstream,
                scores.handstream,
                scores.stamina,
                scores.jackspeed,
                scores.chordjack,
                scores.technical
            );
        }
        println!("]");
    } else {
        let title = if capped {
            "MinaCalc SSR Results (Capped)"
        } else {
            "MinaCalc MSD Results (Uncapped)"
        };
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║  {: <54}  ║", title);
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║  File: {:50} ║", truncate_path(path, 50));
        println!("╠════════╦═════════╦════════╦════════╦════════╦════════════╣");
        println!("║  Rate  ║ Overall ║ Stream ║  Jump  ║  Jack  ║  Technical ║");
        println!("╠════════╬═════════╬════════╬════════╬════════╬════════════╣");

        for (rate, scores) in RATES.iter().zip(all_rates.msds.iter()) {
            println!(
                "║ {:5.2}x ║  {:5.2}  ║ {:5.2}  ║ {:5.2}  ║ {:5.2}  ║   {:5.2}    ║",
                rate,
                scores.overall,
                scores.stream,
                scores.jumpstream,
                scores.jackspeed,
                scores.technical
            );
        }

        println!("╚════════╩═════════╩════════╩════════╩════════╩════════════╝");

        // Highlight 1.0x rate
        let scores_1x = &all_rates.msds[3];
        println!("\n1.0x Summary:");
        print_scores_human(scores_1x, None, capped);
    }

    Ok(())
}

fn print_scores_human(scores: &SkillsetScores, rate: Option<f32>, capped: bool) {
    if let Some(r) = rate {
        println!(
            "Rate: {:.2}x ({})",
            r,
            if capped { "Capped" } else { "Uncapped" }
        );
    }
    println!("  Overall:    {:6.2}", scores.overall);
    println!("  Stream:     {:6.2}", scores.stream);
    println!("  Jumpstream: {:6.2}", scores.jumpstream);
    println!("  Handstream: {:6.2}", scores.handstream);
    println!("  Stamina:    {:6.2}", scores.stamina);
    println!("  JackSpeed:  {:6.2}", scores.jackspeed);
    println!("  Chordjack:  {:6.2}", scores.chordjack);
    println!("  Technical:  {:6.2}", scores.technical);

    // Dominant skillset
    let dominant = get_dominant(scores);
    println!("  Dominant:   {}", dominant);
}

fn get_dominant(s: &SkillsetScores) -> &'static str {
    let skills = [
        (s.stream, "Stream"),
        (s.jumpstream, "Jumpstream"),
        (s.handstream, "Handstream"),
        (s.stamina, "Stamina"),
        (s.jackspeed, "JackSpeed"),
        (s.chordjack, "Chordjack"),
        (s.technical, "Technical"),
    ];
    skills
        .iter()
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .map(|(_, name)| *name)
        .unwrap_or("Unknown")
}

fn truncate_path(path: &Path, max: usize) -> String {
    let s = path.file_name().unwrap_or_default().to_string_lossy();
    if s.len() > max {
        format!("...{}", &s[s.len() - max + 3..])
    } else {
        s.to_string()
    }
}

const RATES: [f32; 14] = [
    0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 2.0,
];
