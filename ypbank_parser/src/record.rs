use crate::errors::{ParseRecordError, ParserError};
use std::fmt::Debug;

/// Основная структура, представляющая транзакционную запись
#[derive(Eq, PartialEq, Debug)]
pub struct Record {
    /// Уникальный идентификатор транзакции.
    pub id: EntityID,
    /// Тип транзакции (Deposit, Transfer, Withdrawal)
    pub tx_type: TxType,
    /// Счёт отправителя; `0` для DEPOSIT.
    pub from_user_id: EntityID,
    /// Счёт получателя; `0` для WITHDRAWAL.
    pub to_user_id: EntityID,
    /// Сумма в наименьшей денежной единице (центах).
    /// Положительное значение для зачислений, отрицательное для списаний.
    pub amount: i64,
    /// Время выполнения транзакции в миллисекундах от эпохи Unix.
    pub timestamp: u64,
    /// Статус транзакции (Success, Failure, Pending)
    pub status: TxStatus,
    /// Текстовое описание транзакции.(Необязательное текстовое описание.)
    ///  bin - Если описание отсутствует, DESC_LEN равен 0.
    ///  csv - Это поле является последним в строке и всегда заключается в двойные кавычки (").
    ///  text - произвольное текстовое описание, UTF-8 в двойныхкавычках.
    pub description: String,
}

impl Record {
    /// Создаёт новую запись транзакции
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
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TxType {
    /// Пополнение счёта (from_user_id = 0)
    Deposit,
    /// Перевод между счетами
    Transfer,
    /// Списание со счёта (to_user_id = 0)
    Withdrawal,
}

impl TxType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TxType::Deposit => "DEPOSIT",
            TxType::Transfer => "TRANSFER",
            TxType::Withdrawal => "WITHDRAWAL",
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
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TxStatus {
    /// Транзакция успешно выполнена
    Success,
    /// Транзакция не выполнена
    Failure,
    /// Транзакция в обработке
    Pending,
}

impl TxStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TxStatus::Success => "SUCCESS",
            TxStatus::Failure => "FAILURE",
            TxStatus::Pending => "PENDING",
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

/// Тип для идентификаторов сущностей (пользователей, транзакций)
pub type EntityID = u64;

#[cfg(test)]
mod tests {
    use crate::record::{TxStatus, TxType};
    #[test]
    fn test_tx_type_to_string() {
        assert_eq!(TxType::Deposit.as_str(), "DEPOSIT");
        assert_eq!(TxType::Transfer.as_str(), "TRANSFER");
        assert_eq!(TxType::Withdrawal.as_str(), "WITHDRAWAL");
    }

    #[test]
    fn test_tx_status_to_string() {
        assert_eq!(TxStatus::Success.as_str(), "SUCCESS");
        assert_eq!(TxStatus::Failure.as_str(), "FAILURE");
        assert_eq!(TxStatus::Pending.as_str(), "PENDING");
    }
}
