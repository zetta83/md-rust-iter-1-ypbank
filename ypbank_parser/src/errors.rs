/// Ошибки, возникающие при парсинге отдельной записи
#[derive(Debug, PartialEq)]
pub enum ParseRecordError {
    /// Неизвестный тип транзакции в данных
    UnknownTxType,
    /// Неизвестный статус транзакции в данных
    UnknownStatus,
}

/// Общие ошибки парсера
#[derive(Debug, PartialEq)]
pub enum ParserError {
    /// Ошибка ввода-вывода
    IoError(String),
    /// Ошибка парсинга числа
    ParseIntError(String),
    /// Ошибка UTF-8 при декодировании строки
    Utf8Error(String),
    /// Ошибка парсинга записи (содержит вложенную ошибку)
    ParseRecord(ParseRecordError),
    /// Неподдерживаемый формат входных данных
    InvalidFormat(String),
    /// Преобразование между указанными форматами не поддерживается
    ConversionNotSupported,
    /// Отсутствует обязательное поле в данных
    RequiredFieldMissing,
    /// Неверный формат данных (магическое число не совпадает)
    InvalidMagicNumber,
    /// Неожиданный конец файла
    UnexpectedEof,
    /// Файл пуст
    EmptyFile,
    /// Ошибка при разборе строки (неверный формат ключ:значение)
    InvalidLineFormat,
    /// Поле не найдено в записи
    FieldNotFound(String),
    /// Неверный размер записи
    InvalidRecordSize,
    /// Файлы имеют разную длину при сравнении
    DifferentLength,
    /// Записи различаются на указанной строке
    DifferentRecord(usize),
}

impl From<ParseRecordError> for ParserError {
    fn from(from: ParseRecordError) -> Self {
        ParserError::ParseRecord(from)
    }
}

/// Реализация `Display` для человекочитаемого вывода ошибок парсинга записи
impl std::fmt::Display for ParseRecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownTxType => write!(f, "Unknown transaction type"),
            Self::UnknownStatus => write!(f, "Unknown status"),
        }
    }
}

impl std::error::Error for ParseRecordError {}

impl From<std::io::Error> for ParserError {
    fn from(err: std::io::Error) -> Self {
        ParserError::IoError(err.to_string())
    }
}

impl From<std::num::ParseIntError> for ParserError {
    fn from(err: std::num::ParseIntError) -> Self {
        ParserError::ParseIntError(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for ParserError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        ParserError::Utf8Error(err.to_string())
    }
}
