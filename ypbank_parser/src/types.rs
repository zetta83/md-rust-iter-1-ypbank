use crate::errors::ParserError;
use crate::record::Record;
use std::io::{BufWriter, Read, Write};

pub trait Parser: Iterator<Item = Result<Record, ParserError>> {
    type Reader: Read;

    fn read_from(reader: Self::Reader) -> Result<Self, ParserError>
    where
        Self: Sized;

    fn write_record<W: Write>(record: &Record, writer: &mut W) -> Result<(), ParserError>;

    fn write_header<W: Write>(_writer: &mut W) -> Result<(), ParserError> {
        Ok(())
    }

    fn write_all_from_iter<I, W>(iter: I, writer: &mut W) -> Result<(), ParserError>
    where
        I: Iterator<Item = Result<Record, ParserError>>,
        W: Write,
    {
        let mut buffer = BufWriter::new(writer);

        for item in iter {
            let record = item?;
            Self::write_record(&record, &mut buffer)?;
        }

        buffer.flush().map_err(|_| ParserError::NotImplemented)?;
        Ok(())
    }

    fn convert_to<OtherParser: Parser, W: Write>(
        &mut self,
        writer: &mut W,
    ) -> Result<(), ParserError> {
        OtherParser::write_header(writer)?;
        OtherParser::write_all_from_iter(self, writer)
    }
}
