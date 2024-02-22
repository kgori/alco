#![allow(unused_imports)]

use std::io::{self, BufReader, BufRead};
use flate2::read::MultiGzDecoder;
use std::fs::File;

mod cli;

pub mod csv {
    use csv::{Reader, ReaderBuilder};
    use flate2::read::MultiGzDecoder;
    use std::fs::File;
    use std::io::{BufReader, Read};
    use std::path::{Path, PathBuf};

    pub type CsvResult = Result<csv::StringRecordsIntoIter<BufReader<MultiGzDecoder<File>>>, csv::Error>;
    pub type CsvReaderResult = Result<Reader<BufReader<MultiGzDecoder<File>>>, csv::Error>;

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

        pub fn reader(&self) -> CsvReaderResult {
            let file = File::open(&self.file_path)?;
            let buf_reader = BufReader::new(MultiGzDecoder::new(file));
            let csv_reader = csv::ReaderBuilder::new()
                .delimiter(b'\t')
                .has_headers(true)
                .from_reader(buf_reader);
            Ok(csv_reader)
        }
    }
}

fn main() -> std::io::Result<()> {
    // Would like to replace the following with `let args = parse_cli()
    let args = cli::parse_cli();

    // Do some work with the args...
    for f in args.filename {
        println!("Hello, {}", f.display());
    }

    let csv_reader = csv::GzCsvReader::new(&args.csv);
    let reader = csv_reader.reader().unwrap();
    for record in reader.into_records().take(10) {
        let record = record.unwrap();
        if &record[4] == "FALSE" {
            for field in &record {
                print!("{},", field)
            }
            println!("");
        }
    }

    Ok(())
}
