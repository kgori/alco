
use csv::Reader;
use flate2::read::MultiGzDecoder;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

pub type GzCsvRecordsResult = Result<csv::StringRecordsIntoIter<BufReader<MultiGzDecoder<File>>>, csv::Error>;
pub type GzCsvReaderResult = Result<Reader<BufReader<MultiGzDecoder<File>>>, csv::Error>;

#[derive(Debug)]
pub struct GzCsvReader {
    pub file_path: PathBuf,
}

impl GzCsvReader {
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        GzCsvReader {
            file_path: file_path.into(),
        }
    }

    pub fn reader(&self) -> GzCsvReaderResult {
        let file = File::open(&self.file_path)?;
        let buf_reader = BufReader::new(MultiGzDecoder::new(file));
        let csv_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .from_reader(buf_reader);
        Ok(csv_reader)
    }

    pub fn records(&self) -> GzCsvRecordsResult {
        let reader = self.reader()?;
        let iterator = reader.into_records();
        Ok(iterator)
    }
}
