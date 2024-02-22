
use csv::Reader;
use flate2::read::MultiGzDecoder;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
#[allow(dead_code)]
pub type LocusFileRecordsResult = Result<csv::StringRecordsIntoIter<BufReader<MultiGzDecoder<File>>>, csv::Error>;
pub type LocusFileReaderResult = Result<Reader<BufReader<MultiGzDecoder<File>>>, csv::Error>;

#[derive(Debug)]
pub struct LocusFileReader {
    pub file_path: PathBuf,
}

impl LocusFileReader {
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        LocusFileReader {
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
