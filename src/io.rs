use csv::Reader;
use flate2::read::MultiGzDecoder;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::error::Error;

pub type LocusFileRecords = csv::StringRecordsIntoIter<BufReader<MultiGzDecoder<File>>>;
pub type LocusFileRecordsResult = Result<LocusFileRecords, csv::Error>;
pub type LocusFileReader = Reader<BufReader<MultiGzDecoder<File>>>;
pub type LocusFileReaderResult = Result<LocusFileReader, csv::Error>;

pub type LocusFileReaderIterator = csv::StringRecordsIntoIter<BufReader<MultiGzDecoder<File>>>;

use crate::errors::AcError;
use crate::variant::Variant;

pub struct LocusBatchIterator {
    iterator: LocusFileReaderIterator,
    peek: Option<Variant>,
    buffer: Vec<Variant>,
    last_seen_chr: Option<String>,
    last_seen_pos: Option<u32>,
    fetch_threshold: Option<u32>,
}

impl LocusBatchIterator {
    pub(crate) fn new(locus_file: LocusFile, fetch_threshold: Option<u32>) -> Result<LocusBatchIterator, Box<dyn Error>> {
        let mut iterator = locus_file.records()?;
        let rec = iterator.next().transpose()?;
        let peek = rec.map(|r| Variant::from_csv_record(&r)).transpose()?;
        Ok(
            Self {
                iterator,
                peek,
                buffer: vec![],
                last_seen_chr: None,
                last_seen_pos: None,
                fetch_threshold,
            }
        )
    }

    fn empty_buffer(&mut self) {
        self.buffer.clear();
        self.last_seen_chr = None;
        self.last_seen_pos = None;
    }

    fn fill_buffer(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            if let Some(var) = &self.peek {
                let chr = var.chr.to_owned();
                let pos: u32 = var.pos_zero_based();
                match (self.last_seen_chr.is_none(), self.last_seen_pos.is_none()) {
                    (true, true) => {
                        self.buffer.push(var.clone());
                        self.last_seen_chr.replace(chr);
                        self.last_seen_pos.replace(pos);
                        self.peek = self.iterator.next().transpose()?
                            .map(|r| Variant::from_csv_record(&r))
                            .transpose()?;
                    }
                    (false, false) => {
                        let chrom_changed = &chr != self.last_seen_chr.as_ref().unwrap();
                        if chrom_changed {
                            self.last_seen_pos = None;
                            self.last_seen_chr = None;
                            break;
                        } else if pos - self.last_seen_pos.unwrap() < self.fetch_threshold.unwrap() {
                            self.buffer.push(var.clone());
                            self.last_seen_chr.replace(chr);
                            self.last_seen_pos.replace(pos);
                            self.peek = self.iterator.next().transpose()?
                                .map(|r| Variant::from_csv_record(&r))
                                .transpose()?;
                        } else {
                            break;
                        }
                    }
                    _ => return Err(Box::new(AcError { message: "Iterator out of sync".to_string() })),
                }
            } else {
                break;
            }
        }
        Ok(())
    }
}

impl Iterator for LocusBatchIterator {
    type Item = Vec<Variant>;


    fn next(&mut self) -> Option<Self::Item> {
        if self.peek.is_none() {
            return None;
        }
        self.fill_buffer().unwrap();

        if self.buffer.is_empty() {
            return None;
        }

        let next_item = self.buffer.clone();
        self.empty_buffer();
        Some(next_item)
    }
}

#[derive(Debug)]
pub struct LocusFile {
    pub file_path: PathBuf,
}

impl LocusFile {
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        LocusFile {
            file_path: file_path.into(),
        }
    }

    pub fn reader(&self) -> LocusFileReaderResult {
        let file = File::open(&self.file_path)?;
        let buf_reader = BufReader::new(MultiGzDecoder::new(file));
        let csv_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .from_reader(buf_reader);
        Ok(csv_reader)
    }

    pub fn records(&self) -> LocusFileRecordsResult {
        let reader = self.reader()?;
        let iterator = reader.into_records();
        Ok(iterator)
    }
}
