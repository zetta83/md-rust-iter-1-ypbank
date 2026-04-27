use clap::Parser as ClapParser;
use std::fs;
use ypbank_parser::{ParserByType, ParserError};

#[derive(ClapParser)]
#[command(
    name = "ypbank_compare",
    version = "0.0.1",
    about = "compare files for different types"
)]
struct Args {
    /// Path file1
    #[arg(long)]
    file1: String,

    /// Format file1 (csv, txt, bin)
    #[arg(long)]
    format1: String,

    /// Path file2
    #[arg(long)]
    file2: String,

    /// Format file2 (csv, txt, bin)
    #[arg(long)]
    format2: String,
}

fn main() -> Result<(), ParserError> {
    let args = Args::parse();

    let mut parser1 = ParserByType::from_format(
        &args.format1,
        fs::File::open(&args.file1).expect("file1 not found"),
    )?;
    let mut parser2 = ParserByType::from_format(
        &args.format2,
        fs::File::open(&args.file2).expect("file2 not found"),
    )?;

    let mut line_num = 0;
    loop {
        match (parser1.next(), parser2.next()) {
            (None, None) => break,
            (Some(Ok(_)), None) | (None, Some(Ok(_))) => {
                println!("files different length");
                return Err(ParserError::NotImplemented);
            }
            (Some(Err(e)), _) | (_, Some(Err(e))) => return Err(e),
            (Some(Ok(r1)), Some(Ok(r2))) => {
                if r1 != r2 {
                    println!("diff {}: {:?} vs {:?}", line_num + 1, r1, r2);
                    return Err(ParserError::NotImplemented);
                }
                line_num += 1;
            }
        }
    }

    println!("The files are identical ({} items)", line_num);
    Ok(())
}
