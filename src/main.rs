extern crate clap;
extern crate rust_htslib;

use rust_htslib::bam;
use rust_htslib::prelude::*;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str;
use std::process;

mod basecounter;
use basecounter::BaseCounter;

mod params;
use params::Params;

#[derive(Debug)]
struct Locus {
    chrom: String,
    position: u32
}

fn get_locus(line: &String) -> Locus {
    let parts: Vec<&str> = line.split('\t').collect();
    let pos = str::parse(parts[1]).expect("Error reading position as integer");
    let loc = Locus{
        chrom: String::from(parts[0]),
        position: pos
    };
    loc
}

fn read_flags_pass(read_flags: u16, required_flags: u16, filter_flags: u16) -> bool {
    return read_flags & filter_flags == 0 && read_flags & required_flags == required_flags;
}

fn main() {
    let params = Params::parse_args();
    if !(params.bamfile.exists()) {
        eprintln!("Bamfile {:?} does not exist.", params.bamfile);
        process::exit(1);
    }
    if !(params.locifile.exists()) {
        eprintln!("Locifile {:?} does not exist.", params.locifile);
        process::exit(1);
    }

    eprintln!("Bam file = {:?}", params.bamfile);
    eprintln!("Loci file = {:?}", params.locifile);
    eprintln!("MinMapQual = {}", params.minmapqual);
    eprintln!("MinBaseQual = {}", params.minbasequal);

    let mut bam_reader = bam::IndexedReader::from_path(&params.bamfile)
        .expect("Error opening bam");
    let bam_header = bam_reader.header().clone();

    let f = File::open(&params.locifile)
        .expect("Error opening locifile");
    let f = BufReader::new(f);
    let lines = f.lines();

    let mut locus: Locus;
    let mut ref_id;
    let mut record: rust_htslib::bam::Record;
    let mut read_pos;

    println!("#CHR\tPOS\tCount_A\tCount_C\tCount_G\tCount_T\tGood_depth");
    for line in lines {
        locus = get_locus(&line.expect("Error reading line from locifile"));

        ref_id = bam_header.tid(locus.chrom.as_bytes()).expect("Error looking up chromosome");
        bam_reader.fetch(ref_id, locus.position - 1, locus.position)
            .expect("Error seeking bam file");

        let mut counter = BaseCounter::new();
        for record_result in bam_reader.records() {
            record = record_result.expect("Error reading record");

            if read_flags_pass(record.flags(), params.required_flag, params.filtered_flag) && record.mapq() >= params.minmapqual {
                read_pos = record.cigar()
                    .read_pos(locus.position - 1, true, true)
                    .expect("Error decoding cigar");

                let base = match read_pos {
                    Some(p) => record.seq()[p as usize] as char,
                    None => '\0',
                };

                let qual = match read_pos {
                    Some(p) => record.qual()[p as usize],
                    None => 0,
                };

                if qual >= params.minbasequal {
                    counter.update(base);
                }
            }
        }
        println!("{}\t{}\t{}", locus.chrom, locus.position, counter.write());
    }
}
