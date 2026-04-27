#[derive(Debug, PartialEq)]
pub enum ParseRecordError {
    UnknownTxType,
    UnknownStatus,
}

#[derive(Debug, PartialEq)]
pub enum ParserError {
    NotImplemented,
    EmptyFile,
    ParseRecord(ParseRecordError),
    InvalidFormat(String),
    ConversionNotSupported,
    RequiredFieldMissing,
}

impl From<ParseRecordError> for ParserError {
    fn from(from: ParseRecordError) -> Self {
        ParserError::ParseRecord(from)
    }
}

impl std::fmt::Display for ParseRecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownTxType => write!(f, "Unknown transaction type"),
            Self::UnknownStatus => write!(f, "Unknown status"),
        }
    }
}

impl std::error::Error for ParseRecordError {}
