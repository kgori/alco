#![allow(unused_imports)]

use std::io::BufRead;

mod cli;
mod csv;

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
