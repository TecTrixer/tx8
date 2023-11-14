use clap::Parser;
use std::fs::read;
use std::process::exit;
use tx8_core::*;

#[derive(Parser)]
#[command(name = "tx8 Interpreter")]
#[command(author = "TecTrixer")]
#[command(version = "0.1.0")]
#[command(about = "This interpreter takes tx8 ROM files (.txr) and executes them.")]
struct Cli {
    filename: String,
}

fn main() -> Result<(), ()> {
    let filename = Cli::parse().filename;
    println!("Reading {filename}");
    let file = match read(&filename) {
        Ok(d) => d,
        Err(e) => {
            println!("Failed to open \"{filename}\": {e:?}");
            return Ok(());
        }
    };
    match run_code(file) {
        Ok(_) => exit(0),
        Err(e) => {
            println!("\n{}", e);
            exit(1)
        }
    }
}
