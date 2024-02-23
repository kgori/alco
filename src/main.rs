use std::error::Error;
use rust_htslib::bam::{self, Read};
use core::ops::Index;

mod cli;
mod io;

mod base_counter;
mod errors;

use base_counter::BaseCounter;
use io::Variant;

fn process_batch(bam: &mut bam::IndexedReader, positions: &Vec<Variant>) -> Result<(), Box<dyn Error>> {
    if positions.is_empty() {
        return Ok(());
    }

    // Determine the minimum and maximum positions
    let chr = positions.first().map(|p| &p.chr).unwrap();
    let min_pos = positions.first().map(|p| p.pos_zero_based()).unwrap();
    let max_pos = positions.last().map(|p| p.pos_one_based()).unwrap();

    bam.fetch((chr, min_pos, max_pos))?;

    let mut pileup = bam.pileup();
    let mut pileup_col = pileup.next().ok_or("No next position in pileup")??;

    // Process pileup for each CSV record in the batch
    for position in positions.iter() {
        let ref_id = &position.chr;
        let pos_0based = position.pos_zero_based();
        let pos_1based = position.pos_one_based();
        let refbase = position.refbase;
        let altbase = position.altbase;

        let mut counts = BaseCounter::new(refbase, altbase);

        if pileup_col.pos() > pos_0based {
            continue;
        }

        while pileup_col.pos() < pos_0based {
            pileup_col = pileup.next().ok_or("No next position in pileup")??;
        }

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
        println!("{ref_id}\t{pos_1based}\t{refbase}\t{altbase}\t{counts}");
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::parse_cli();
    let mut bam = bam::IndexedReader::from_path(&args.bamfile)?;
    let rdr = io::LocusFile::new(&args.locifile);
    let itr = io::LocusBatchIterator::new(rdr, args.fetch_threshold)?;

    println!("CHROM\tPOS\tREF\tALT\tA\tC\tG\tT\tNREF\tNALT");
    for batch in itr {
        let _ = process_batch(&mut bam, &batch);
    }
    Ok(())
}
