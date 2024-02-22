use clap::{CommandFactory, Parser};
use clap::error::ErrorKind;
use std::{fs, path::PathBuf};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct ProgramOptions {
    #[arg(short, long, num_args(2))]
    pub filename: Vec<PathBuf>,

    #[arg(short, long)]
    pub csv: PathBuf,
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
    for filename in &args.filename {
        validate_file(filename);
    }
    validate_file(&args.csv);
    args
}
