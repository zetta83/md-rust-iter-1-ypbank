use crate::Parser;
use crate::errors::ParserError;
use crate::record::{Record, TxStatus, TxType};
use std::collections::HashMap;
use std::fmt::Display;
use std::io::{BufRead, BufReader, Read, Write};

type TxtRecord = Record;

/// Отображение записи в текстовый формат KEY: VALUE для реализации `Display`
pub struct TxtDisplay<'a>(pub &'a TxtRecord);

impl<'a> Display for TxtDisplay<'a> {
    /// Форматирует запись как многострочный текст в формате "ПОЛЕ: ЗНАЧЕНИЕ".
    /// Записи разделяются пустой строкой.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let item = self.0;

        writeln!(f, "TX_ID: {}", item.id)?;
        writeln!(f, "TX_TYPE: {}", item.tx_type.as_str())?;
        writeln!(f, "FROM_USER_ID: {}", item.from_user_id)?;
        writeln!(f, "TO_USER_ID: {}", item.to_user_id)?;
        writeln!(f, "AMOUNT: {}", item.amount)?;
        writeln!(f, "TIMESTAMP: {}", item.timestamp)?;
        writeln!(f, "STATUS: {}", item.status.as_str())?;
        writeln!(f, "DESCRIPTION: {}", item.description)?;
        writeln!(f)?; // empty line
        Ok(())
    }
}

impl TxtRecord {
    fn as_txt_record(&self) -> String {
        TxtDisplay(self).to_string()
    }
}

/// Парсер текстового формата.
/// Читает записи в формате "ПОЛЕ: ЗНАЧЕНИЕ", разделённые пустыми строками.
/// Строки, начинающиеся с '#', игнорируются как комментарии.
#[derive(Debug)]
pub struct ParserTxt<R: Read> {
    reader: BufReader<R>,
}

impl<R: Read> Parser for ParserTxt<R> {
    /// Буферизованный читатель входных данных
    type Reader = R;

    fn read_from(reader: R) -> Result<Self, ParserError> {
        let mut buf_reader = BufReader::new(reader);
        let mut header_line = String::new();

        match buf_reader.read_line(&mut header_line) {
            Ok(0) => Err(ParserError::EmptyFile), // empty file
            Ok(_) => Ok(ParserTxt { reader: buf_reader }),
            Err(e) => Err(ParserError::IoError(e.to_string())),
        }
    }

    fn write_record<W: Write>(record: &Record, writer: &mut W) -> Result<(), ParserError> {
        writer
            .write(record.as_txt_record().as_ref())
            .map_err(ParserError::from)?;
        Ok(())
    }
}

impl<R: Read> Iterator for ParserTxt<R> {
    type Item = Result<TxtRecord, ParserError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut record_map = HashMap::new();
        for line in self.reader.by_ref().lines() {
            match line {
                Ok(line) => {
                    let line = line.trim();

                    if line.starts_with('#') {
                        continue;
                    }

                    if line.is_empty() {
                        break;
                    }

                    if let Some((key, value)) = line.split_once(':') {
                        let key = key.trim();
                        let value = value.trim();
                        record_map.insert(key.to_string(), value.to_string());
                    } else {
                        return Some(Err(ParserError::InvalidLineFormat));
                    };
                }
                Err(e) => return Some(Err(ParserError::IoError(e.to_string()))),
            }
        }

        if record_map.is_empty() {
            return None;
        }

        match (|| -> Result<TxtRecord, ParserError> {
            let tx_id = record_map
                .remove("TX_ID")
                .ok_or_else(|| ParserError::FieldNotFound("TX_ID".to_string()))?
                .parse::<u64>()?;

            let tx_type = TxType::try_from(
                record_map
                    .remove("TX_TYPE")
                    .ok_or_else(|| ParserError::FieldNotFound("TX_TYPE".to_string()))?
                    .as_str(),
            )?;

            let from_user_id = record_map
                .remove("FROM_USER_ID")
                .ok_or_else(|| ParserError::FieldNotFound("FROM_USER_ID".to_string()))?
                .parse::<u64>()?;

            let to_user_id = record_map
                .remove("TO_USER_ID")
                .ok_or_else(|| ParserError::FieldNotFound("TO_USER_ID".to_string()))?
                .parse::<u64>()?;

            let amount = record_map
                .remove("AMOUNT")
                .ok_or_else(|| ParserError::FieldNotFound("AMOUNT".to_string()))?
                .parse::<i64>()?;

            let timestamp = record_map
                .remove("TIMESTAMP")
                .ok_or_else(|| ParserError::FieldNotFound("TIMESTAMP".to_string()))?
                .parse::<u64>()?;

            let status = TxStatus::try_from(
                record_map
                    .remove("STATUS")
                    .ok_or_else(|| ParserError::FieldNotFound("STATUS".to_string()))?
                    .as_str(),
            )?;

            let description = record_map
                .remove("DESCRIPTION")
                .ok_or_else(|| ParserError::FieldNotFound("DESCRIPTION".to_string()))?;

            Ok(TxtRecord::new(
                tx_id,
                tx_type,
                from_user_id,
                to_user_id,
                amount,
                timestamp,
                status,
                description.as_str(),
            ))
        })() {
            Ok(result) => Some(Ok(result)),
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{TxStatus, TxType};
    use std::io::{BufWriter, Cursor};
    use time::macros::datetime;

    #[test]
    fn test_parse() -> Result<(), ParserError> {
        let data = b"";
        let mut cursor = Cursor::new(&data[..]);
        let buffer = BufReader::new(&mut cursor);

        assert!(matches!(
            ParserTxt::read_from(buffer),
            Err(ParserError::EmptyFile)
        ));

        let data = r#"# Record 1 (DEPOSIT)
TX_TYPE: DEPOSIT
TO_USER_ID: 9223372036854775807
FROM_USER_ID: 0
TIMESTAMP: 1633036860000
DESCRIPTION: "Record number 1"
TX_ID: 1000000000000000
AMOUNT: 100
STATUS: FAILURE

# Record 2 (TRANSFER)
DESCRIPTION: "Record number 2"
TIMESTAMP: 1633036920000
STATUS: PENDING
AMOUNT: 200
TX_ID: 1000000000000001
TX_TYPE: TRANSFER
FROM_USER_ID: 9223372036854775807
TO_USER_ID: 9223372036854775807

"#;
        let mut cursor = Cursor::new(data);
        let buffer = BufReader::new(&mut cursor);

        let mut parser = ParserTxt::read_from(buffer)?;

        let tx_1 = parser.next().ok_or(ParserError::EmptyFile)??;
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

        let tx_2 = parser.next().ok_or(ParserError::EmptyFile)??;
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

        Ok(())
    }

    #[test]
    fn test_write_record() -> Result<(), ParserError> {
        let mut buffer = Vec::new();

        let record = TxtRecord::new(
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
                ParserTxt::<std::io::Empty>::write_record(&record, &mut buff_writer),
                Ok(())
            )
        }

        assert_eq!(
            String::from_utf8(buffer).map_err(ParserError::from)?,
            r#"TX_ID: 1000000000000000
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9223372036854775807
AMOUNT: 100
TIMESTAMP: 1633036860000
STATUS: FAILURE
DESCRIPTION: "Record number 1"

"#
        );

        Ok(())
    }

    #[test]
    fn test_record_txt_as_txt_record() {
        assert_eq!(
            TxtRecord::new(
                1000000000000000,
                TxType::Deposit,
                0,
                9223372036854775807,
                100,
                (datetime!(2021-09-30 21:21:00 UTC).unix_timestamp() as u64) * 1000,
                TxStatus::Failure,
                "\"Record number 1\"",
            )
            .as_txt_record(),
            r#"TX_ID: 1000000000000000
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9223372036854775807
AMOUNT: 100
TIMESTAMP: 1633036860000
STATUS: FAILURE
DESCRIPTION: "Record number 1"

"#
        )
    }
}
