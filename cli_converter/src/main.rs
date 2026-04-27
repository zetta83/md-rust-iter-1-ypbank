use clap::Parser as ClapParser;
use std::{fs, io};
use ypbank_parser::{ParserByType, ParserError};

#[derive(ClapParser)]
#[command(
    name = "ypbank_converter",
    version = "0.0.1",
    about = "converter of your program"
)]
struct Args {
    /// Path input file
    #[arg(long)]
    input: String,

    /// Format input file (csv, txt, bin)
    #[arg(long)]
    input_format: String,

    /// Format output file (csv, txt, bin)
    #[arg(long)]
    output_format: String,
}

fn main() -> Result<(), ParserError> {
    let args = Args::parse();

    ParserByType::from_format(
        &args.input_format,
        fs::File::open(&args.input).expect("input file not found"),
    )?
    .convert_to(&args.output_format, &mut io::stdout())?;

    Ok(())
}
