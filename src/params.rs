// All command line argument handling is done here

use clap::{App, Arg};
use std::path::PathBuf;
use std::process;

pub struct Params {
    pub bamfile: PathBuf,
    pub locifile: PathBuf,
    pub minmapqual: u8,
    pub minbasequal: u8,
    pub flags: u16,
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
                .short("q")
                .long("minbasequal")
                .takes_value(true)
                .value_name("INT")
                .default_value("35")
                .help("Minimum base quality"))
            .arg(Arg::with_name("minmapqual")
                .short("m")
                .long("minmapqual")
                .takes_value(true)
                .value_name("INT")
                .default_value("20")
                .help("Minimum mapping quality"))
            .arg(Arg::with_name("flags")
                .short("f")
                .long("flags")
                .takes_value(true)
                .value_name("INT")
                .default_value("1796")
                .help("Reads matching this flag combination will be ignored (default=1796: NOT secondary, NOT optical duplicate, NOT unmapped, NOT qc failed"))
            .get_matches();

        let bamfile = matches.value_of("bamfile").unwrap();
        let locifile = matches.value_of("locifile").unwrap();
        let minmapqual = str::parse(matches.value_of("minmapqual").unwrap())
            .unwrap_or_else(|e| {
                println!("Error parsing minmapqual as an integer: {:?}", e);
                process::exit(1);
            });

        let minbasequal = str::parse(matches.value_of("minbasequal").unwrap())
            .unwrap_or_else(|e| {
                println!("Error parsing minbasequal as an integer: {:?}", e);
                process::exit(1);
            });

        let flags = str::parse(matches.value_of("flags").unwrap())
            .unwrap_or_else(|e| {
                println!("Error parsing flags as an integer: {:?}", e);
                process::exit(1);
            });

        Params {
            bamfile: PathBuf::from(bamfile),
            locifile: PathBuf::from(locifile),
            minmapqual: minmapqual,
            minbasequal: minbasequal,
            flags: flags
        }
    }
}