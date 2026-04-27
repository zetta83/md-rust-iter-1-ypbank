use crate::Parser;
use crate::errors::ParserError;
use crate::record::{Record, TxStatus, TxType};
use std::fmt::{Display, Formatter};
use std::io::{BufRead, BufReader, Read, Write};

const HEADER_FIELDS: &str =
    "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n";

type CsvRecord = Record;

pub struct CsvDisplay<'a>(pub &'a CsvRecord);

impl<'a> Display for CsvDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let item = self.0;

        writeln!(
            f,
            "{}",
            [
                item.id.to_string(),
                String::from(item.tx_type.clone()),
                item.from_user_id.to_string(),
                item.to_user_id.to_string(),
                item.amount.to_string(),
                item.timestamp.to_string(),
                String::from(item.status.clone()),
                item.description.to_string(),
            ]
            .join(",")
        )
    }
}

impl CsvRecord {
    fn as_csv_record(&self) -> String {
        CsvDisplay(self).to_string()
    }
}

#[derive(Debug, Clone, Default)]
struct FieldMapping {
    id_idx: Option<usize>,
    tx_type_idx: Option<usize>,
    from_user_id_idx: Option<usize>,
    to_user_id_idx: Option<usize>,
    amount_idx: Option<usize>,
    timestamp_idx: Option<usize>,
    status_idx: Option<usize>,
    description_idx: Option<usize>,
}

impl FieldMapping {
    fn default() -> Self {
        FieldMapping {
            id_idx: None,
            tx_type_idx: None,
            from_user_id_idx: None,
            to_user_id_idx: None,
            amount_idx: None,
            timestamp_idx: None,
            status_idx: None,
            description_idx: None,
        }
    }

    fn from_header(header: &str) -> Result<Self, ParserError> {
        let fields: Vec<_> = header
            .trim()
            .split(',')
            .map(|v| v.trim().to_lowercase())
            .collect();

        let mut mapping = FieldMapping::default();

        // TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
        for (idx, field) in fields.iter().enumerate() {
            match field.as_str() {
                "tx_id" => mapping.id_idx = Some(idx),
                "tx_type" => mapping.tx_type_idx = Some(idx),
                "from_user_id" => mapping.from_user_id_idx = Some(idx),
                "to_user_id" => mapping.to_user_id_idx = Some(idx),
                "amount" => mapping.amount_idx = Some(idx),
                "timestamp" => mapping.timestamp_idx = Some(idx),
                "status" => mapping.status_idx = Some(idx),
                "description" => mapping.description_idx = Some(idx),
                _ => {}
            }
        }

        if mapping.id_idx.is_none() {
            return Err(ParserError::RequiredFieldMissing);
        }

        Ok(mapping)
    }

    fn parse_record(&self, fields: &[&str]) -> Result<CsvRecord, ParserError> {
        Ok(CsvRecord::new(
            self.parse_u64_field(fields, self.id_idx)?,
            self.parse_tx_type_field(fields, self.tx_type_idx)?,
            self.parse_u64_field(fields, self.from_user_id_idx)?,
            self.parse_u64_field(fields, self.to_user_id_idx)?,
            self.parse_i64_field(fields, self.amount_idx)?,
            self.parse_u64_field(fields, self.timestamp_idx)?,
            self.parse_status_field(fields, self.status_idx)?,
            &self.parse_string_field(fields, self.description_idx)?,
        ))
    }

    fn parse_u64_field(&self, fields: &[&str], idx: Option<usize>) -> Result<u64, ParserError> {
        let idx = idx.ok_or(ParserError::NotImplemented)?;
        let field = fields.get(idx).ok_or(ParserError::NotImplemented)?;
        field
            .parse::<u64>()
            .map_err(|_| ParserError::NotImplemented)
    }

    fn parse_i64_field(&self, fields: &[&str], idx: Option<usize>) -> Result<i64, ParserError> {
        let idx = idx.ok_or(ParserError::NotImplemented)?;
        let field = fields.get(idx).ok_or(ParserError::NotImplemented)?;
        field
            .parse::<i64>()
            .map_err(|_| ParserError::NotImplemented)
    }

    fn parse_tx_type_field(
        &self,
        fields: &[&str],
        idx: Option<usize>,
    ) -> Result<TxType, ParserError> {
        let idx = idx.ok_or(ParserError::NotImplemented)?;
        let field = fields.get(idx).ok_or(ParserError::NotImplemented)?;
        TxType::try_from(*field).map_err(|_| ParserError::NotImplemented)
    }

    fn parse_status_field(
        &self,
        fields: &[&str],
        idx: Option<usize>,
    ) -> Result<TxStatus, ParserError> {
        let idx = idx.ok_or(ParserError::NotImplemented)?;
        let field = fields.get(idx).ok_or(ParserError::NotImplemented)?;
        TxStatus::try_from(*field).map_err(|_| ParserError::NotImplemented)
    }

    fn parse_string_field(
        &self,
        fields: &[&str],
        idx: Option<usize>,
    ) -> Result<String, ParserError> {
        let idx = idx.ok_or(ParserError::NotImplemented)?;
        let field = fields.get(idx).ok_or(ParserError::NotImplemented)?;
        field
            .parse::<String>()
            .map_err(|_| ParserError::NotImplemented)
    }
}

pub struct ParserCsv<R: Read> {
    reader: BufReader<R>,
    mapping: FieldMapping,
    is_header_read: bool,
}

impl<R: Read> ParserCsv<R> {
    pub fn new(reader: R) -> Self {
        ParserCsv {
            reader: BufReader::new(reader),
            mapping: FieldMapping::default(),
            is_header_read: false,
        }
    }
}

impl<R: Read> Parser for ParserCsv<R> {
    type Reader = R;

    fn read_from(reader: R) -> Result<Self, ParserError> {
        let mut buf_reader = BufReader::new(reader);
        let mut header_line = String::new();

        match buf_reader.read_line(&mut header_line) {
            Ok(0) => Err(ParserError::EmptyFile), // empty file
            Ok(_) => {
                let mapping = FieldMapping::from_header(&header_line)?;
                Ok(ParserCsv {
                    reader: buf_reader,
                    mapping,
                    is_header_read: true,
                })
            }
            Err(_) => Err(ParserError::NotImplemented),
        }
    }

    fn write_record<W: Write>(record: &Record, writer: &mut W) -> Result<(), ParserError> {
        writer
            .write(record.as_csv_record().as_ref())
            .map_err(|_| ParserError::NotImplemented)?;
        Ok(())
    }

    fn write_header<W: Write>(writer: &mut W) -> Result<(), ParserError> {
        writer
            .write(HEADER_FIELDS.as_ref())
            .map_err(|_| ParserError::NotImplemented)?;
        Ok(())
    }
}

impl<R: Read> Iterator for ParserCsv<R> {
    type Item = Result<CsvRecord, ParserError>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_header_read {
            return None;
        }

        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => None, // end of file
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    return self.next(); // skip empty line
                }

                let fields: Vec<_> = line.split(',').collect();

                match self.mapping.parse_record(&fields) {
                    Ok(record) => Some(Ok(record)),
                    Err(err) => Some(Err(err)),
                }
            }
            Err(_) => Some(Err(ParserError::NotImplemented)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::csv_format::{CsvRecord, HEADER_FIELDS, ParserCsv};
    use crate::record::{TxStatus, TxType};
    use crate::{Parser, ParserError};
    use std::io::{BufReader, BufWriter, Cursor, Write};
    use time::macros::datetime;

    #[test]
    fn test_read_from_empty_file() {
        let data = b"";
        let mut cursor = Cursor::new(&data[..]);
        let buffer = BufReader::new(&mut cursor);

        assert!(matches!(
            ParserCsv::read_from(buffer),
            Err(ParserError::EmptyFile)
        ));
    }

    #[test]
    fn test_read_from_empty_header() {
        let data = r#"1000000000000000,DEPOSIT,0,9223372036854775807,100,1633036860000,FAILURE,"Record number 1"#;
        let mut cursor = Cursor::new(data);
        let buffer = BufReader::new(&mut cursor);

        assert!(matches!(
            ParserCsv::read_from(buffer),
            Err(ParserError::RequiredFieldMissing)
        ));
    }

    #[test]
    fn test_read_from() {
        let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1000000000000000,DEPOSIT,0,9223372036854775807,100,1633036860000,FAILURE,"Record number 1"
1000000000000001,TRANSFER,9223372036854775807,9223372036854775807,200,1633036920000,PENDING,"Record number 2"
"#;

        let mut cursor = Cursor::new(data);
        let buffer = BufReader::new(&mut cursor);
        let mut parser = ParserCsv::read_from(buffer).unwrap();

        let tx_1 = parser.next().unwrap().unwrap();
        assert_eq!(tx_1.id, 1000000000000000);
        assert_eq!(tx_1.tx_type, TxType::Deposit);
        assert_eq!(tx_1.from_user_id, 0);
        assert_eq!(tx_1.to_user_id, 9223372036854775807);
        assert_eq!(tx_1.amount, 100);
        assert_eq!(
            tx_1.timestamp,
            (datetime!(2021-09-30 21:21:00 UTC).unix_timestamp() as u64) * 1000
        );
        assert_eq!(tx_1.status, TxStatus::Failure);
        assert_eq!(tx_1.description, "\"Record number 1\"");

        let tx_2 = parser.next().unwrap().unwrap();
        assert_eq!(tx_2.id, 1000000000000001);
        assert_eq!(tx_2.tx_type, TxType::Transfer);
        assert_eq!(tx_2.from_user_id, 9223372036854775807);
        assert_eq!(tx_2.to_user_id, 9223372036854775807);
        assert_eq!(tx_2.amount, 200);
        assert_eq!(
            tx_2.timestamp,
            (datetime!(2021-09-30 21:22:00 UTC).unix_timestamp() as u64) * 1000
        );
        assert_eq!(tx_2.status, TxStatus::Pending);
        assert_eq!(tx_2.description, "\"Record number 2\"");
    }

    #[test]
    fn test_record_as_csv() {
        assert_eq!(
            CsvRecord::new(
                1000000000000000,
                TxType::Deposit,
                0,
                9223372036854775807,
                100,
                (datetime!(2021-09-30 21:21:00 UTC).unix_timestamp() as u64) * 1000,
                TxStatus::Failure,
                "\"Record number 1\"",
            )
            .as_csv_record(),
            "1000000000000000,DEPOSIT,0,9223372036854775807,100,1633036860000,FAILURE,\"Record number 1\"\n"
        );
    }

    #[test]
    fn test_write_header() {
        let mut buffer = Vec::new();

        {
            let mut cursor = Cursor::new(&mut buffer);
            let mut buff_writer = BufWriter::new(&mut cursor);

            assert_eq!(
                ParserCsv::<std::io::Empty>::write_header(&mut buff_writer),
                Ok(())
            );

            buff_writer.flush().unwrap();
        }

        assert_eq!(String::from_utf8(buffer).unwrap(), HEADER_FIELDS);
    }

    #[test]
    fn test_write_record() {
        let mut buffer = Vec::new();

        let record = CsvRecord::new(
            1000000000000000,
            TxType::Deposit,
            0,
            9223372036854775807,
            100,
            (datetime!(2021-09-30 21:21:00 UTC).unix_timestamp() as u64) * 1000,
            TxStatus::Failure,
            "\"Record number 1\"",
        );

        {
            let mut cursor = Cursor::new(&mut buffer);
            let mut buff_writer = BufWriter::new(&mut cursor);
            assert_eq!(
                ParserCsv::<std::io::Empty>::write_record(&record, &mut buff_writer),
                Ok(())
            )
        }

        assert_eq!(
            String::from_utf8(buffer).unwrap(),
            "1000000000000000,DEPOSIT,0,9223372036854775807,100,1633036860000,FAILURE,\"Record number 1\"\n"
        );
    }
}
