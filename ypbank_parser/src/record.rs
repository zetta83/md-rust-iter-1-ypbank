use crate::errors::{ParseRecordError, ParserError};
use std::fmt::Debug;

#[derive(Eq, PartialEq, Debug)]
pub struct Record {
    /// Уникальный идентификатор транзакции.
    pub id: EntityID,
    /// Тип транзакции
    pub tx_type: TxType,
    /// Счёт отправителя; `0` для DEPOSIT.
    pub from_user_id: EntityID,
    /// Счёт получателя; `0` для WITHDRAWAL.
    pub to_user_id: EntityID,
    /// Сумма в наименьшей денежной единице (центах). Положительное значение для зачислений, отрицательное для списаний.
    pub amount: i64,
    /// Время выполнения транзакции в миллисекундах от эпохи Unix.
    pub timestamp: u64,
    /// Статус транзакции
    pub status: TxStatus,
    /// Текстовое описание транзакции.(Необязательное текстовое описание.)
    ///  bin - Если описание отсутствует, DESC_LEN равен 0.
    ///  csv - Это поле является последним в строке и всегда заключается в двойные кавычки (").
    ///  text - произвольное текстовое описание, UTF-8 в двойныхкавычках.
    pub description: String,
}

impl Record {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: EntityID,
        tx_type: TxType,
        from_user_id: EntityID,
        to_user_id: EntityID,
        amount: i64,
        timestamp: u64,
        status: TxStatus,
        description: &str,
    ) -> Self {
        Record {
            id,
            tx_type,
            from_user_id,
            to_user_id,
            amount,
            timestamp,
            status,
            description: description.to_string(),
        }
    }
}

/// Тип транзакции
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TxType {
    Deposit,
    Transfer,
    Withdrawal,
}

impl From<TxType> for String {
    fn from(tx_type: TxType) -> Self {
        match tx_type {
            TxType::Deposit => "Deposit".to_uppercase(),
            TxType::Transfer => "Transfer".to_uppercase(),
            TxType::Withdrawal => "Withdrawal".to_uppercase(),
        }
    }
}

impl TryFrom<&str> for TxType {
    type Error = ParserError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "deposit" => Ok(TxType::Deposit),
            "transfer" => Ok(TxType::Transfer),
            "withdrawal" => Ok(TxType::Withdrawal),
            _ => Err(ParserError::from(ParseRecordError::UnknownTxType)),
        }
    }
}

/// Статус транзакции
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TxStatus {
    Success,
    Failure,
    Pending,
}

impl From<TxStatus> for String {
    fn from(status: TxStatus) -> Self {
        match status {
            TxStatus::Success => "Success".to_uppercase(),
            TxStatus::Failure => "Failure".to_uppercase(),
            TxStatus::Pending => "Pending".to_uppercase(),
        }
    }
}

impl TryFrom<&str> for TxStatus {
    type Error = ParserError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "success" => Ok(TxStatus::Success),
            "failure" => Ok(TxStatus::Failure),
            "pending" => Ok(TxStatus::Pending),
            _ => Err(ParserError::from(ParseRecordError::UnknownStatus)),
        }
    }
}

pub type EntityID = u64;

#[cfg(test)]
mod tests {
    use crate::record::{TxStatus, TxType};
    #[test]
    fn test_tx_type_to_string() {
        assert_eq!(String::from(TxType::Deposit), "DEPOSIT");
        assert_eq!(String::from(TxType::Transfer), "TRANSFER");
        assert_eq!(String::from(TxType::Withdrawal), "WITHDRAWAL");
    }

    #[test]
    fn test_tx_status_to_string() {
        assert_eq!(String::from(TxStatus::Success), "SUCCESS");
        assert_eq!(String::from(TxStatus::Failure), "FAILURE");
        assert_eq!(String::from(TxStatus::Pending), "PENDING");
    }
}
