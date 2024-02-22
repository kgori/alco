#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]

use std::error::Error;
use std::io::BufRead;
use rust_htslib::bam::{self, Read, IndexedReader, pileup::Pileups};
use csv::StringRecord;
use core::ops::Index;

mod cli;
mod io;

mod base_counter;
use base_counter::BaseCounter;

fn process_record(bam: &mut bam::IndexedReader, csv_record: ::csv::StringRecord) -> Result<(), Box<dyn Error>> {
    let ref_id = csv_record.get(0).ok_or("Missing CHROM in CSV record")?.to_owned();
    let pos_1based: u32 = csv_record.get(1).ok_or("Missing POS in CSV record")?.parse()?;
    let pos_0based = pos_1based - 1;
    let refbase = csv_record.get(2).ok_or("Missing REF in CSV record")?.chars().nth(0).unwrap();
    let altbase = csv_record.get(3).ok_or("Missing ALT in CSV record")?.chars().nth(0).unwrap();

    let mut counts = BaseCounter::new(refbase, altbase);

    bam.fetch((&ref_id, pos_0based, pos_1based))?;

    let pileup = bam.pileup();
    for p in pileup {
        let pileup_col = p?;
        if pileup_col.pos() == pos_0based {
            for aln in pileup_col.alignments() {
                let mapping_quality = aln.record().mapq();
                let qpos = aln.qpos();
                if let Some(i) = qpos {
                    let base_qual = aln.record().qual()[i];
                    let base = *aln.record().seq().index(i) as char;
                    if mapping_quality > 30 && base_qual > 15 {
                        counts.add(base);
                    }
                }
            }
        }
    }
    println!("{ref_id}\t{pos_1based}\t{refbase}\t{altbase}\t{counts}");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Would like to replace the following with `let args = parse_cli()
    let args = cli::parse_cli();

    let csv_reader = io::LocusFileReader::new(&args.locifile)
        .reader()?;
    let mut bam = bam::IndexedReader::from_path(&args.bamfile)?;
    let header = bam.header().clone();

    let mut curr_ref = std::str::from_utf8(header.tid2name(0))?.to_owned();
    bam.fetch(&curr_ref)?;
    //let mut pileup = bam.pileup();

    for record in csv_reader.into_records().take(1000) {
        process_record(&mut bam, record?)?;
    }
    Ok(())
}
