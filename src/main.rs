#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::error::Error;
use std::io::BufRead;
use rust_htslib::bam::{self, Read, IndexedReader, pileup::Pileups};
use csv::StringRecord;
use core::ops::Index;

mod cli;
mod io;

mod base_counter;
mod errors;

use base_counter::BaseCounter;

fn process_batch(bam: &mut bam::IndexedReader, csv_records: &Vec<csv::StringRecord>) -> Result<(), Box<dyn Error>> {
    // Collect positions from CSV records
    let mut positions: Vec<(String, u32, u32, char, char)> = Vec::new();
    for csv_record in csv_records {
        let ref_id = csv_record.get(0).ok_or("Missing CHROM in CSV record")?.to_owned();
        let pos_1based: u32 = csv_record.get(1).ok_or("Missing POS in CSV record")?.parse()?;
        let pos_0based = pos_1based - 1;
        let refbase = csv_record.get(2).ok_or("Missing REF in CSV record")?.chars().next().unwrap();
        let altbase = csv_record.get(3).ok_or("Missing ALT in CSV record")?.chars().next().unwrap();
        positions.push((ref_id, pos_0based, pos_1based, refbase, altbase));
    }

    // Determine the minimum and maximum positions
    let min_pos = *positions.iter().map(|(_, pos, _, _, _)| pos).min().unwrap();
    let max_pos = *positions.iter().map(|(_, _, pos, _, _)| pos).max().unwrap();

    bam.fetch((&positions[0].0, min_pos, max_pos))?;

    let mut pileup = bam.pileup();
    let mut pileup_col = pileup.next().ok_or("No next position in pileup")??;

    // Process pileup for each CSV record in the batch
    for (ref_id, pos_0based, pos_1based, refbase, altbase) in positions.iter() {
        let mut counts = BaseCounter::new(*refbase, *altbase);

        if pileup_col.pos() > *pos_0based {
            continue;
        }

        while pileup_col.pos() < *pos_0based {
            pileup_col = pileup.next().ok_or("No next position in pileup")??;
        }

        if pileup_col.pos() == *pos_0based {
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
        println!("{ref_id}\t{pos_1based}\t{refbase}\t{altbase}\t{counts}");
    }

    Ok(())
}

fn process_record(bam: &mut bam::IndexedReader, csv_record: csv::StringRecord) -> Result<(), Box<dyn Error>> {
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

    let csv_reader = io::LocusFile::new(&args.locifile)
        .reader()?;
    let mut bam = bam::IndexedReader::from_path(&args.bamfile)?;
    let header = bam.header().clone();

    let mut curr_ref = std::str::from_utf8(header.tid2name(0))?.to_owned();
    bam.fetch(&curr_ref)?;

    let vec_records: Vec<csv::StringRecord> = io::LocusFile::new(&args.locifile)
        .records()?
        .take(10)
        .collect::<Result<Vec<_>, _>>()?;

    println!("CHROM\tPOS\tREF\tALT\tA\tC\tG\tT\tNREF\tNALT");
    let _ = process_batch(&mut bam, &vec_records);

    let rdr = io::LocusFile::new(&args.locifile);
    let mut itr = io::LocusBatchIterator::new(rdr)?;
    for batch in itr {
        eprintln!("{:#?} {:#?}", batch.first(), batch.last());
        let _ = process_batch(&mut bam, &batch);
    }

    Ok(())
}
