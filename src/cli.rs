use clap::{CommandFactory, Parser};
use clap::error::ErrorKind;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct ProgramOptions {
    #[arg(short, long)]
    pub bamfile: PathBuf,

    #[arg(short, long)]
    pub locifile: PathBuf,

    #[arg(short, long, default_value = "35")]
    pub minmapqual: u8,

    #[arg(short = 'q', long, default_value = "20")]
    pub minbasequal: u8,

    #[arg(short = 'f', long, default_value = "3")]
    pub required_flag: u16,

    #[arg(short = 'F', long, default_value = "3852")]
    pub filtered_flag: u16,

    #[arg(short = 'g', long, default_value = "1000000")]
    pub fetch_threshold: Option<u32>,
}

fn validate_file(file: &Path) {
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
    validate_file(&args.bamfile);
    validate_file(&args.locifile);
    args
}
