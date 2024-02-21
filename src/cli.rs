use clap::{CommandFactory, Parser};
use clap::error::ErrorKind;
use std::{fs, path::PathBuf};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct ProgramOptions {
    #[arg(short, long)]
    pub filename: PathBuf,
}

fn validate_file(file: &PathBuf) {
    if !file.exists() {
        let mut cmd = ProgramOptions::command();
        cmd.error(
            ErrorKind::ValueValidation,
            format!(
                "file `{}` not found",
                file.display()
            ),
        ).exit();
    }
}

pub fn parse_cli() -> ProgramOptions {
    let args = ProgramOptions::parse();
    validate_file(&args.filename);
    args
}
