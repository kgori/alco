#![allow(unused_imports)]

mod cli;

fn main() {
    // Would like to replace the following with `let args = parse_cli()
    let args = cli::parse_cli();

    // Do some work with the args...
    println!("Hello, {}!", args.filename.display());
}
