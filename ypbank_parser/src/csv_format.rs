use crate::Parser;
use crate::errors::ParserError;
use crate::record::{Record, TxStatus, TxType};
use std::fmt::{Display, Formatter};
use std::io::{BufRead, BufReader, Read, Write};

/// Строка заголовка CSV-файла с именами всех полей
const HEADER_FIELDS: &str =
    "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n";

type CsvRecord = Record;

/// Отображение записи в CSV-формат для реализации `Display`
pub struct CsvDisplay<'a>(pub &'a CsvRecord);

impl<'a> Display for CsvDisplay<'a> {
    /// Форматирует запись как строку CSV, разделённую запятыми
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let item = self.0;

        writeln!(
            f,
            "{}",
            [
                item.id.to_string(),
                item.tx_type.as_str().to_string(),
                item.from_user_id.to_string(),
                item.to_user_id.to_string(),
                item.amount.to_string(),
                item.timestamp.to_string(),
                item.status.as_str().to_string(),
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

/// Отображение полей CSV на индексы колонок.
/// Позволяет читать CSV с полями в любом порядке.
#[derive(Debug, Clone, Default)]
struct FieldMapping {
    /// Индекс колонки TX_ID (обязательное поле)
    id_idx: Option<usize>,
    /// Индекс колонки TX_TYPE
    tx_type_idx: Option<usize>,
    /// Индекс колонки FROM_USER_ID
    from_user_id_idx: Option<usize>,
    /// Индекс колонки TO_USER_ID
    to_user_id_idx: Option<usize>,
    /// Индекс колонки AMOUNT
    amount_idx: Option<usize>,
    /// Индекс колонки TIMESTAMP
    timestamp_idx: Option<usize>,
    /// Индекс колонки STATUS
    status_idx: Option<usize>,
    /// Индекс колонки DESCRIPTION
    description_idx: Option<usize>,
}

impl FieldMapping {
    /// Создаёт пустое отображение со всеми полями `None`
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

    /// Создаёт отображение из строки заголовка CSV.
    /// Ищет поля по имени (регистронезависимо).
    ///
    /// # Errors
    /// Возвращает `ParserError::RequiredFieldMissing`, если отсутствует поле `TX_ID`
    fn from_header(header: &str) -> Result<Self, ParserError> {
        let fields: Vec<_> = header
            .trim()
            .split(',')
            .map(|v| v.trim().to_lowercase())
            .collect();
        let mut mapping = FieldMapping::default();

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

    /// Парсит запись из массива полей согласно отображению
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

    /// Парсит 64-битное беззнаковое целое из указанного поля
    fn parse_u64_field(&self, fields: &[&str], idx: Option<usize>) -> Result<u64, ParserError> {
        let idx = idx.ok_or(ParserError::FieldNotFound("index not found".to_string()))?;
        let field = fields.get(idx).ok_or(ParserError::FieldNotFound(format!(
            "field at index {}",
            idx
        )))?;
        field.parse::<u64>().map_err(ParserError::from)
    }

    /// Парсит 64-битное знаковое целое из указанного поля
    fn parse_i64_field(&self, fields: &[&str], idx: Option<usize>) -> Result<i64, ParserError> {
        let idx = idx.ok_or(ParserError::FieldNotFound("index not found".to_string()))?;
        let field = fields.get(idx).ok_or(ParserError::FieldNotFound(format!(
            "field at index {}",
            idx
        )))?;
        field.parse::<i64>().map_err(ParserError::from)
    }

    /// Парсит тип транзакции из строкового поля
    fn parse_tx_type_field(
        &self,
        fields: &[&str],
        idx: Option<usize>,
    ) -> Result<TxType, ParserError> {
        let idx = idx.ok_or(ParserError::FieldNotFound("index not found".to_string()))?;
        let field = fields.get(idx).ok_or(ParserError::FieldNotFound(format!(
            "field at index {}",
            idx
        )))?;
        TxType::try_from(*field)
    }

    /// Парсит статус транзакции из строкового поля
    fn parse_status_field(
        &self,
        fields: &[&str],
        idx: Option<usize>,
    ) -> Result<TxStatus, ParserError> {
        let idx = idx.ok_or(ParserError::FieldNotFound("index not found".to_string()))?;
        let field = fields.get(idx).ok_or(ParserError::FieldNotFound(format!(
            "field at index {}",
            idx
        )))?;
        TxStatus::try_from(*field)
    }

    /// Парсит строковое поле
    fn parse_string_field(
        &self,
        fields: &[&str],
        idx: Option<usize>,
    ) -> Result<String, ParserError> {
        let idx = idx.ok_or(ParserError::FieldNotFound("index not found".to_string()))?;
        let field = fields.get(idx).ok_or(ParserError::FieldNotFound(format!(
            "field at index {}",
            idx
        )))?;
        Ok(field.to_string())
    }
}

/// Парсер CSV-формата.
/// Читает CSV с заголовком и строками данных.
pub struct ParserCsv<R: Read> {
    /// Буферизованный читатель входных данных
    reader: BufReader<R>,
    /// Отображение полей на индексы колонок
    mapping: FieldMapping,
    /// Флаг, указывающий, был ли уже прочитан заголовок
    is_header_read: bool,
}

impl<R: Read> ParserCsv<R> {
    /// Создаёт новый экземпляр парсера CSV (без чтения заголовка)
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
            Err(e) => Err(ParserError::IoError(e.to_string())),
        }
    }

    fn write_record<W: Write>(record: &Record, writer: &mut W) -> Result<(), ParserError> {
        writer
            .write(record.as_csv_record().as_ref())
            .map_err(ParserError::from)?;
        Ok(())
    }

    fn write_header<W: Write>(writer: &mut W) -> Result<(), ParserError> {
        writer
            .write(HEADER_FIELDS.as_ref())
            .map_err(ParserError::from)?;
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
            Err(e) => Some(Err(ParserError::IoError(e.to_string()))),
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
    fn test_read_from() -> Result<(), ParserError> {
        let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1000000000000000,DEPOSIT,0,9223372036854775807,100,1633036860000,FAILURE,"Record number 1"
1000000000000001,TRANSFER,9223372036854775807,9223372036854775807,200,1633036920000,PENDING,"Record number 2"
"#;

        let mut cursor = Cursor::new(data);
        let buffer = BufReader::new(&mut cursor);
        let mut parser = ParserCsv::read_from(buffer)?;

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
    fn test_write_header() -> Result<(), ParserError> {
        let mut buffer = Vec::new();

        {
            let mut cursor = Cursor::new(&mut buffer);
            let mut buff_writer = BufWriter::new(&mut cursor);

            assert_eq!(
                ParserCsv::<std::io::Empty>::write_header(&mut buff_writer),
                Ok(())
            );

            buff_writer.flush().map_err(ParserError::from)?;
        }

        assert_eq!(
            String::from_utf8(buffer).map_err(ParserError::from)?,
            HEADER_FIELDS
        );
        Ok(())
    }

    #[test]
    fn test_write_record() -> Result<(), ParserError> {
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
            String::from_utf8(buffer).map_err(ParserError::from)?,
            "1000000000000000,DEPOSIT,0,9223372036854775807,100,1633036860000,FAILURE,\"Record number 1\"\n"
        );

        Ok(())
    }
}
