// All command line argument handling is done here

use clap::{App, Arg};
use std::path::PathBuf;
use std::process;

pub struct Params {
    pub bamfile: PathBuf,
    pub locifile: PathBuf,
    pub minmapqual: u8,
    pub minbasequal: u8,
    pub filtered_flag: u16,
    pub required_flag: u16,
}

impl Params {
    pub fn parse_args() -> Params {
        let matches = App::new("AlCo")
            .version("1.0.0")
            .author("Kevin Gori, kcg25@cam.ac.uk")
            .about("Basic alleleCounter clone")
            .arg(Arg::with_name("bamfile")
                .short("b")
                .long("bamfile")
                .takes_value(true)
                .value_name("FILE")
                .required(true)
                .help("Input BAM file"))
            .arg(Arg::with_name("locifile")
                .short("l")
                .long("locifile")
                .takes_value(true)
                .value_name("FILE")
                .required(true)
                .help("Input loci file"))
            .arg(Arg::with_name("minbasequal")
                .short("m")
                .long("minbasequal")
                .takes_value(true)
                .value_name("INT")
                .default_value("20")
                .help("Minimum base quality"))
            .arg(Arg::with_name("minmapqual")
                .short("q")
                .long("minmapqual")
                .takes_value(true)
                .value_name("INT")
                .default_value("35")
                .help("Minimum mapping quality"))
            .arg(Arg::with_name("required_flag")
                .short("f")
                .long("required-flag")
                .takes_value(true)
                .value_name("INT")
                .default_value("3")
                .help("Reads must match this flag combination to be counted (default=3: read paired, read mapped in proper pair"))
            .arg(Arg::with_name("filtered_flag")
                .short("F")
                .long("filtered-flag")
                .takes_value(true)
                .value_name("INT")
                .default_value("3852")
                .help("Reads matching this flag combination will be ignored (default=3852: NOT read unmapped, NOT mate unmapped, NOT secondary, NOT qc failed, NOT optical duplicate, NOT supplementary"))
            .get_matches();

        let bamfile = matches.value_of("bamfile").unwrap();
        let locifile = matches.value_of("locifile").unwrap();
        let minmapqual = str::parse(matches.value_of("minmapqual").unwrap())
            .unwrap_or_else(|e| {
                eprintln!("Error parsing minmapqual as an integer: {:?}", e);
                process::exit(1);
            });

        let minbasequal = str::parse(matches.value_of("minbasequal").unwrap())
            .unwrap_or_else(|e| {
                eprintln!("Error parsing minbasequal as an integer: {:?}", e);
                process::exit(1);
            });

        let filtered_flag = str::parse(matches.value_of("filtered_flag").unwrap())
            .unwrap_or_else(|e| {
                eprintln!("Error parsing flags as an integer: {:?}", e);
                process::exit(1);
            });

        let required_flag = str::parse(matches.value_of("required_flag").unwrap())
            .unwrap_or_else(|e| {
                eprintln!("Error parsing flags as an integer: {:?}", e);
                process::exit(1);
            });

        Params {
            bamfile: PathBuf::from(bamfile),
            locifile: PathBuf::from(locifile),
            minmapqual: minmapqual,
            minbasequal: minbasequal,
            filtered_flag: filtered_flag,
            required_flag: required_flag
        }
    }
}
