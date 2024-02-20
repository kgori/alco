#![allow(unused_imports)]
use clap::{Parser, CommandFactory};
use std::{fs, path::PathBuf};
use clap::error::ErrorKind;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    filename: PathBuf,
}

fn parse_cli() -> Cli {
    let args = Cli::parse();
    if !args.filename.exists() {
        let mut cmd = Cli::command();
        cmd.error(
            ErrorKind::ValueValidation,
            format!(
                "file `{}` not found",
                args.filename.display()
            ),
        ).exit();
    }
    args
}

fn main() {
    // Would like to replace the following with `let args = parse_cli()
    let args = parse_cli();

    // Do some work with the args...
    println!("Hello, {}!", args.filename.display());
}
