use rust_htslib::bam;
use std::error::Error;

mod base_counter;
mod cli;
mod errors;
mod io;
mod processing;
mod variant;

use processing::process_batch;

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::parse_cli();
    let mut bam = bam::IndexedReader::from_path(&args.bamfile)?;
    let locus_file = io::LocusFile::new(&args.locifile);
    let batch_iterator = io::LocusBatchIterator::new(locus_file, args.fetch_threshold)?;

    println!("CHROM\tPOS\tREF\tALT\tTOTAL\tA\tC\tG\tT\tNREF\tNALT");
    for batch in batch_iterator {
        process_batch(&mut bam, &batch, &args)?;
    }
    Ok(())
}
