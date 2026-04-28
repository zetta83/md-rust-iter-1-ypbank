use crate::errors::ParserError;
use crate::record::Record;
use crate::{Parser, ParserBin, ParserCsv, ParserTxt};
use std::io::{Read, Write};

/// Тип-перечисление, объединяющий различные реализации парсеров (CSV, TXT, BIN).
/// Позволяет единообразно работать с разными форматами входных данных.
pub enum ParserByType<R: Read> {
    /// Вариант для CSV-парсера
    Csv(ParserCsv<R>),
    /// Вариант для TXT-парсера
    Txt(ParserTxt<R>),
    /// Вариант для бинарного парсера
    Bin(ParserBin<R>),
}

impl<R: Read> ParserByType<R> {
    /// Создаёт экземпляр `ParserByType` на основе указанного формата.
    ///
    /// # Arguments
    /// * `format` - Строка, определяющая формат: "csv", "txt" или "bin"
    /// * `reader` - Объект, реализующий трейт `Read`, откуда будут читаться данные
    ///
    /// # Errors
    /// Возвращает `ParserError::InvalidFormat`, если передан неподдерживаемый формат
    pub fn from_format(format: &str, reader: R) -> Result<Self, ParserError> {
        match format {
            "csv" => Ok(ParserByType::Csv(ParserCsv::read_from(reader)?)),
            "txt" => Ok(ParserByType::Txt(ParserTxt::read_from(reader)?)),
            "bin" => Ok(ParserByType::Bin(ParserBin::read_from(reader)?)),
            _ => Err(ParserError::InvalidFormat(format.to_string())),
        }
    }

    /// Преобразует данные из текущего формата в целевой формат и записывает их в `writer`.
    ///
    /// # Arguments
    /// * `output_format` - Целевой формат ("csv", "txt" или "bin")
    /// * `writer` - Объект, реализующий `Write`, куда будут записаны преобразованные данные
    ///
    /// # Errors
    /// Возвращает `ParserError::ConversionNotSupported`, если преобразование между указанными
    /// форматами невозможно
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

/// Реализация трейта `Parser` для `ParserByType`.
///
/// # Panics
/// Методы `read_from` и `write_record` не реализованы и вызовут панику при использовании.
/// Вместо них следует использовать `ParserByType::from_format()` для создания экземпляра.
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

/// Реализация `Iterator` для `ParserByType`.
/// Позволяет последовательно получать записи (`Record`) из парсера в зависимости от выбранного формата.
///
/// # Yields
/// * `Some(Ok(Record))` - успешно прочитанная запись
/// * `Some(Err(ParserError))` - ошибка при чтении записи
/// * `None` - достигнут конец данных
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
