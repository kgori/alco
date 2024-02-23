use csv::Reader;
use flate2::read::MultiGzDecoder;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::error::Error;
use std::fmt;

#[allow(dead_code)]
pub type LocusFileRecords = csv::StringRecordsIntoIter<BufReader<MultiGzDecoder<File>>>;
pub type LocusFileRecordsResult = Result<LocusFileRecords, csv::Error>;
pub type LocusFileReader = Reader<BufReader<MultiGzDecoder<File>>>;
pub type LocusFileReaderResult = Result<LocusFileReader, csv::Error>;

pub type LocusFileReaderIterator = csv::StringRecordsIntoIter<BufReader<MultiGzDecoder<File>>>;

const GULF: u32 = 1000000;

use crate::errors::AcError;

pub struct LocusBatchIterator {
    iterator: LocusFileReaderIterator,
    peek: Option<csv::StringRecord>,
    buffer: Vec<csv::StringRecord>,
    last_seen_chr: Option<String>,
    last_seen_pos: Option<u32>,
    fetch_threshold: Option<u32>,
}

impl LocusBatchIterator {
    pub(crate) fn new(locus_file: LocusFile, fetch_threshold: Option<u32>) -> Result<LocusBatchIterator, Box<dyn Error>> {
        let mut iterator = locus_file.records()?;
        let peek = iterator.next().transpose()?;
        Ok(
            Self {
                iterator: iterator,
                peek: peek,
                buffer: vec![],
                last_seen_chr: None,
                last_seen_pos: None,
                fetch_threshold: fetch_threshold,
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
            if let Some(record) = &self.peek {
                let chr = record.get(0).ok_or("Missing first field")?.to_owned();
                let pos: u32 = record.get(1).ok_or("Missing second field")?.parse()?;
                match (self.last_seen_chr.is_none(), self.last_seen_pos.is_none()) {
                    (true, true) => {
                        self.buffer.push(record.clone());
                        self.last_seen_chr.replace(chr);
                        self.last_seen_pos.replace(pos);
                        self.peek = self.iterator.next().transpose()?
                    }
                    (false, false) => {
                        let chrom_changed = &chr != self.last_seen_chr.as_ref().unwrap();
                        if chrom_changed {
                            self.last_seen_pos = None;
                            self.last_seen_chr = None;
                            break;
                        } else if pos - self.last_seen_pos.unwrap() < self.fetch_threshold.or(Some(GULF)).unwrap() {
                            self.buffer.push(record.clone());
                            self.last_seen_chr.replace(chr);
                            self.last_seen_pos.replace(pos);
                            self.peek = self.iterator.next().transpose()?
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
    type Item = Vec<csv::StringRecord>;


    fn next(&mut self) -> Option<Self::Item> {
        if self.peek.is_none() {
            return None;
        }
        let _ = self.fill_buffer().unwrap();

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

    #[allow(dead_code)]
    pub fn records(&self) -> LocusFileRecordsResult {
        let reader = self.reader()?;
        let iterator = reader.into_records();
        Ok(iterator)
    }
}
