use crate::errors::ParserError;
use crate::record::Record;
use crate::{Parser, ParserBin, ParserCsv, ParserTxt};
use std::io::{Read, Write};

pub enum ParserByType<R: Read> {
    Csv(ParserCsv<R>),
    Txt(ParserTxt<R>),
    Bin(ParserBin<R>),
}

impl<R: Read> ParserByType<R> {
    pub fn from_format(format: &str, reader: R) -> Result<Self, ParserError> {
        match format {
            "csv" => Ok(ParserByType::Csv(ParserCsv::read_from(reader)?)),
            "txt" => Ok(ParserByType::Txt(ParserTxt::read_from(reader)?)),
            "bin" => Ok(ParserByType::Bin(ParserBin::read_from(reader)?)),
            _ => Err(ParserError::InvalidFormat(format.to_string())),
        }
    }

    pub fn convert_to<W: Write>(
        &mut self,
        output_format: &str,
        writer: &mut W,
    ) -> Result<(), ParserError> {
        match (self, output_format) {
            (ParserByType::Csv(parser), "txt") => parser.convert_to::<ParserTxt<R>, _>(writer),
            (ParserByType::Csv(parser), "bin") => parser.convert_to::<ParserBin<R>, _>(writer),
            (ParserByType::Csv(parser), "csv") => parser.convert_to::<ParserCsv<R>, _>(writer),
            (ParserByType::Txt(parser), "csv") => parser.convert_to::<ParserCsv<R>, _>(writer),
            (ParserByType::Txt(parser), "bin") => parser.convert_to::<ParserBin<R>, _>(writer),
            (ParserByType::Txt(parser), "txt") => parser.convert_to::<ParserTxt<R>, _>(writer),
            (ParserByType::Bin(parser), "csv") => parser.convert_to::<ParserCsv<R>, _>(writer),
            (ParserByType::Bin(parser), "txt") => parser.convert_to::<ParserTxt<R>, _>(writer),
            (ParserByType::Bin(parser), "bin") => parser.convert_to::<ParserBin<R>, _>(writer),
            _ => Err(ParserError::ConversionNotSupported),
        }
    }
}

impl<R: Read> Parser for ParserByType<R> {
    type Reader = R;

    fn read_from(_: Self::Reader) -> Result<Self, ParserError>
    where
        Self: Sized,
    {
        unimplemented!("use instead ParserType::from_format()")
    }

    fn write_record<W: Write>(_: &Record, _: &mut W) -> Result<(), ParserError> {
        unimplemented!()
    }
}

impl<R: Read> Iterator for ParserByType<R> {
    type Item = Result<Record, ParserError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ParserByType::Csv(parser) => parser.next(),
            ParserByType::Txt(parser) => parser.next(),
            ParserByType::Bin(parser) => parser.next(),
        }
    }
}
