extern crate clap;
extern crate rust_htslib;

use clap::{App,Arg};

use rust_htslib::bam;
use rust_htslib::prelude::*;

use std::path::PathBuf;
use std::str;
use std::process;

pub struct Params {
    pub bamfile: PathBuf,
    pub locifile: PathBuf,
    pub minmapqual: u32,
    pub minbasequal: u32,
}

fn parse_args() -> Params {
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
        .get_matches();

    let bamfile = matches.value_of("bamfile").unwrap();
    let locifile = matches.value_of("locifile").unwrap();
    let minmapqual: u32 = str::parse(matches.value_of("minmapqual").unwrap())
        .unwrap_or_else(|e| {
            println!("Error parsing minmapqual as an integer: {:?}", e);
            process::exit(1);
        });

    let minbasequal: u32 = str::parse(matches.value_of("minbasequal").unwrap())
        .unwrap_or_else(|e| {
            println!("Error parsing minbasequal as an integer: {:?}", e);
            process::exit(1);
        });

    Params{
        bamfile: PathBuf::from(bamfile),
        locifile: PathBuf::from(locifile),
        minmapqual: minmapqual,
        minbasequal: minbasequal
    }
}


fn main() {
    let params = parse_args();
    if !(params.bamfile.exists()) {
        println!("Bamfile {:?} does not exist.", params.bamfile);
        process::exit(1);
    }
    if !(params.locifile.exists()) {
        println!("Locifile {:?} does not exist.", params.locifile);
        process::exit(1);
    }

    println!("Bam file = {:?}", params.bamfile);
    println!("Loci file = {:?}", params.locifile);
    println!("MinMapQual = {}", params.minmapqual);
    println!("MinBaseQual = {}", params.minbasequal);

    let mut bam = bam::Reader::from_path(&params.bamfile).unwrap();

	for p in bam.pileup() {
		let pileup = p.unwrap();
		println!("{}:{} depth {}", pileup.tid(), pileup.pos(), pileup.depth());

		for alignment in pileup.alignments() {
			if !alignment.is_del() && !alignment.is_refskip() {
                // println!("{:?}", str::from_utf8(&alignment.record().seq().as_bytes()).unwrap());
                println!("Base {}", alignment.record().seq()[alignment.qpos().unwrap()] as char);
			}
            // // mark indel start
			// match alignment.indel() {
				// bam::pileup::Indel::Ins(len) => println!("Insertion of length {} between this and next position.", len),
				// bam::pileup::Indel::Del(len) => println!("Deletion of length {} between this and next position.", len),
				// bam::pileup::Indel::None => ()
			// }
		}
	}
}
