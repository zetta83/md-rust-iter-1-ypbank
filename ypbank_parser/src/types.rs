use crate::errors::ParserError;
use crate::record::Record;
use std::io::{BufWriter, Read, Write};

/// Основной трейт для всех парсеров форматов.
/// Объединяет функциональность чтения, записи и конвертации записей.
pub trait Parser: Iterator<Item = Result<Record, ParserError>> {
    /// Тип читателя, из которого парсер читает данные
    type Reader: Read;

    /// Создаёт экземпляр парсера из читателя
    fn read_from(reader: Self::Reader) -> Result<Self, ParserError>
    where
        Self: Sized;

    /// Записывает одну запись в указанный писатель
    fn write_record<W: Write>(record: &Record, writer: &mut W) -> Result<(), ParserError>;

    /// Записывает заголовок формата (если требуется).
    /// По умолчанию не делает ничего.
    fn write_header<W: Write>(_writer: &mut W) -> Result<(), ParserError> {
        Ok(())
    }

    /// Записывает все записи из итератора в указанный писатель.
    /// Использует буферизованный вывод для эффективности.
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

        buffer.flush().map_err(ParserError::from)?;
        Ok(())
    }

    /// Преобразует текущий парсер в другой формат.
    /// Записывает заголовок целевого формата, затем все записи.
    fn convert_to<OtherParser: Parser, W: Write>(
        &mut self,
        writer: &mut W,
    ) -> Result<(), ParserError> {
        OtherParser::write_header(writer)?;
        OtherParser::write_all_from_iter(self, writer)
    }
}
