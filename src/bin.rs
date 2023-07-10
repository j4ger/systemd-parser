use clap::Parser;
use systemd_parser::parse_file;

#[derive(Parser, Debug)]
struct CliOptions {
    /// Path to input file
    input: String,
}

fn main() {
    let args = CliOptions::parse();
    match parse_file(args.input) {
        Err(err) => {
            eprintln!("Error occured: {}", err);
        }
        Ok(result) => {
            println!("{:#?}", result);
        }
    }
}
