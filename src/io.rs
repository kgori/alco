use csv::Reader;
use flate2::read::MultiGzDecoder;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::PathBuf;

pub type LocusFileRecords = csv::StringRecordsIntoIter<BufReader<Box<dyn Read>>>;
pub type LocusFileRecordsResult = Result<LocusFileRecords, csv::Error>;
pub type LocusFileReader = Reader<BufReader<Box<dyn Read>>>;
pub type LocusFileReaderResult = Result<LocusFileReader, csv::Error>;

pub type LocusFileReaderIterator = csv::StringRecordsIntoIter<BufReader<Box<dyn Read>>>;

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
    pub(crate) fn new(
        locus_file: LocusFile,
        fetch_threshold: Option<u32>,
    ) -> Result<LocusBatchIterator, Box<dyn Error>> {
        let mut iterator = locus_file.records()?;
        let rec = iterator.next().transpose()?;
        let peek = rec.map(|r| Variant::from_csv_record(&r)).transpose()?;
        Ok(Self {
            iterator,
            peek,
            buffer: vec![],
            last_seen_chr: None,
            last_seen_pos: None,
            fetch_threshold,
        })
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
                        self.peek = self
                            .iterator
                            .next()
                            .transpose()?
                            .map(|r| Variant::from_csv_record(&r))
                            .transpose()?;
                    }
                    (false, false) => {
                        let chrom_changed = &chr != self.last_seen_chr.as_ref().unwrap();
                        if chrom_changed {
                            self.last_seen_pos = None;
                            self.last_seen_chr = None;
                            break;
                        } else if pos - self.last_seen_pos.unwrap() < self.fetch_threshold.unwrap()
                        {
                            self.buffer.push(var.clone());
                            self.last_seen_chr.replace(chr);
                            self.last_seen_pos.replace(pos);
                            self.peek = self
                                .iterator
                                .next()
                                .transpose()?
                                .map(|r| Variant::from_csv_record(&r))
                                .transpose()?;
                        } else {
                            break;
                        }
                    }
                    _ => {
                        return Err(Box::new(AcError {
                            message: "Iterator out of sync".to_string(),
                        }));
                    }
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

    fn open_reader(&self) -> Result<Box<dyn Read>, std::io::Error> {
        let mut file = File::open(&self.file_path)?;

        // Sniff first two bytes, looking for gzip signature
        let mut buf = [0u8; 2];
        let n = file.read(&mut buf)?;
        file.rewind()?;

        let reader: Box<dyn Read> = if n == 2 && buf == [0x1f, 0x8b] {
            Box::new(MultiGzDecoder::new(file))
        } else {
            Box::new(file)
        };

        Ok(reader)
    }

    pub fn reader(&self) -> LocusFileReaderResult {
        let reader = self.open_reader()?;
        let buf_reader = BufReader::new(reader);
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

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_plain_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "chr\tpos\tref\talt").unwrap();
        writeln!(file, "1\t100\tA\tG").unwrap();
        writeln!(file, "1\t200\tC\tT").unwrap();
        file
    }

    fn write_gzipped_csv() -> NamedTempFile {
        let file = NamedTempFile::new().unwrap();
        {
            let mut encoder = GzEncoder::new(&file, Compression::default());
            writeln!(encoder, "chr\tpos\tref\talt").unwrap();
            writeln!(encoder, "1\t100\tA\tG").unwrap();
            writeln!(encoder, "1\t200\tC\tT").unwrap();
            writeln!(encoder, "2\t300\tG\tA").unwrap();
            encoder.finish().unwrap();
        }
        file
    }

    #[test]
    fn test_read_plain_csv() {
        let file = write_plain_csv();
        let locus_file = LocusFile::new(file.path());
        let mut records = locus_file.records().unwrap();

        let r1 = records.next().unwrap().unwrap();
        assert_eq!(r1.get(0).unwrap(), "1");
        assert_eq!(r1.get(1).unwrap(), "100");
        assert_eq!(r1.get(2).unwrap(), "A");
        assert_eq!(r1.get(3).unwrap(), "G");

        let r2 = records.next().unwrap().unwrap();
        assert_eq!(r2.get(0).unwrap(), "1");
        assert_eq!(r2.get(1).unwrap(), "200");
        assert_eq!(r2.get(2).unwrap(), "C");
        assert_eq!(r2.get(3).unwrap(), "T");

        assert!(records.next().is_none());
    }

    #[test]
    fn test_read_gzipped_csv() {
        let file = write_gzipped_csv();
        let locus_file = LocusFile::new(file.path());
        let mut records = locus_file.records().unwrap();

        let r1 = records.next().unwrap().unwrap();
        assert_eq!(r1.get(0).unwrap(), "1");
        assert_eq!(r1.get(1).unwrap(), "100");
        assert_eq!(r1.get(2).unwrap(), "A");
        assert_eq!(r1.get(3).unwrap(), "G");

        let r2 = records.next().unwrap().unwrap();
        assert_eq!(r2.get(0).unwrap(), "1");
        assert_eq!(r2.get(1).unwrap(), "200");
        assert_eq!(r2.get(2).unwrap(), "C");
        assert_eq!(r2.get(3).unwrap(), "T");

        let r3 = records.next().unwrap().unwrap();
        assert_eq!(r3.get(0).unwrap(), "2");
        assert_eq!(r3.get(1).unwrap(), "300");
        assert_eq!(r3.get(2).unwrap(), "G");
        assert_eq!(r3.get(3).unwrap(), "A");

        assert!(records.next().is_none());
    }
}
