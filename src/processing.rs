use std::error::Error;
use rust_htslib::bam::{self, Read};
use crate::base_counter::BaseCounter;
use crate::variant::Variant;
use core::ops::Index;
use crate::errors::AcError;
use crate::cli::ProgramOptions;

pub fn process_batch(bam: &mut bam::IndexedReader, positions: &Vec<Variant>, args: &ProgramOptions) -> Result<(), Box<dyn Error>> {
    if positions.is_empty() {
        return Err(Box::new(AcError { message: "Wasn't expecting an empty batch...".to_string() }))
    }

    // Determine the minimum and maximum positions
    let chr = positions.first().map(|p| &p.chr).unwrap();
    let min_pos = positions.first().map(|p| p.pos_zero_based()).unwrap();
    let max_pos = positions.last().map(|p| p.pos_one_based()).unwrap();

    bam.fetch((chr, min_pos, max_pos))?;
    let mut pileup = bam.pileup();

    // Set this to true if the pileup iterator gets exhausted at any point
    // It signals to print zero counts at all remaining batch positions
    let mut drain = false;

    if let Some(result) = pileup.next() {
        let mut pileup_col = result?;

        // Process pileup for each CSV record in the batch
        for position in positions.iter() {
            let ref_id = &position.chr;
            let pos_0based = position.pos_zero_based();
            let pos_1based = position.pos_one_based();
            let refbase = position.refbase;
            let altbase = position.altbase;
            let mut counts = BaseCounter::new(refbase, altbase);

            if drain || pileup_col.pos() > pos_0based {
                println!("{ref_id}\t{pos_1based}\t{refbase}\t{altbase}\t{counts}");
                continue;
            }

            while pileup_col.pos() < pos_0based {
                let peek = pileup.next();
                if let Some(more_data) = peek {
                    pileup_col = more_data?;
                }
                else {
                    drain = true;
                    assert!(pileup_col.pos() != pos_0based);
                    break;
                }
            }

            if pileup_col.pos() == pos_0based {
                for aln in pileup_col.alignments() {
                    let mapping_quality = aln.record().mapq();
                    let qpos = aln.qpos();
                    let flags = aln.record().flags();
                    let flags_pass = flags & args.required_flag != 0 && flags & args.filtered_flag == 0;
                    if let Some(i) = qpos {
                        let base_qual = aln.record().qual()[i];
                        let base = *aln.record().seq().index(i) as char;
                        let quals_pass = mapping_quality >= args.minmapqual && base_qual >= args.minbasequal;
                        if flags_pass && quals_pass {
                            counts.add(base);
                        }
                    }
                }
            }
            println!("{ref_id}\t{pos_1based}\t{refbase}\t{altbase}\t{counts}");
        }
    } else {
        for position in positions.iter() {
            let ref_id = &position.chr;
            let pos_1based = position.pos_one_based();
            let refbase = position.refbase;
            let altbase = position.altbase;
            let counts = BaseCounter::new(refbase, altbase);
            println!("{ref_id}\t{pos_1based}\t{refbase}\t{altbase}\t{counts}");
        }
    }
    Ok(())
}
