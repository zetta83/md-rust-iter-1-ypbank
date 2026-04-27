use crate::Parser;
use crate::errors::{ParseRecordError, ParserError};
use crate::record::{Record, TxStatus, TxType};
use std::io::{BufReader, Read, Write};
use std::iter::Iterator;

const MAGIC_HEADER: u32 = 0x59_50_42_4E; // 0x59 0x50 0x42 0x4E = 'YPBN',

type BinRecord = Record;

struct HeaderBinRecord {
    magic: u32,
    record_size: u32,
}

impl HeaderBinRecord {
    fn to_vec(&self) -> Vec<u8> {
        [
            self.magic.to_be_bytes().to_vec(),
            self.record_size.to_be_bytes().to_vec(),
        ]
        .concat()
    }
}

impl TryFrom<u8> for TxType {
    type Error = ParseRecordError;
    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            0 => Ok(TxType::Deposit),
            1 => Ok(TxType::Transfer),
            2 => Ok(TxType::Withdrawal),
            _ => Err(ParseRecordError::UnknownTxType),
        }
    }
}

impl From<TxType> for u8 {
    fn from(tx_type: TxType) -> u8 {
        match tx_type {
            TxType::Deposit => 0,
            TxType::Transfer => 1,
            TxType::Withdrawal => 2,
        }
    }
}

impl TryFrom<u8> for TxStatus {
    type Error = ParseRecordError;
    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            0 => Ok(TxStatus::Success),
            1 => Ok(TxStatus::Failure),
            2 => Ok(TxStatus::Pending),
            _ => Err(ParseRecordError::UnknownStatus),
        }
    }
}

impl From<TxStatus> for u8 {
    fn from(status: TxStatus) -> u8 {
        match status {
            TxStatus::Success => 0,
            TxStatus::Failure => 1,
            TxStatus::Pending => 2,
        }
    }
}

impl BinRecord {
    fn make_header(&self, body: &Vec<u8>) -> HeaderBinRecord {
        HeaderBinRecord {
            magic: MAGIC_HEADER,
            record_size: body.len() as u32,
        }
    }

    fn make_body(&self) -> Vec<u8> {
        [
            self.id.to_be_bytes().to_vec(),                         // TX_ID | u64
            vec![u8::from(self.tx_type.clone())],                   // TX_TYPE | u8
            self.from_user_id.to_be_bytes().to_vec(),               // FROM_USER_ID | u64
            self.to_user_id.to_be_bytes().to_vec(),                 // TO_USER_ID | u64
            self.amount.to_be_bytes().to_vec(),                     // AMOUNT | i64
            self.timestamp.to_be_bytes().to_vec(),                  // TIMESTAMP | u64
            vec![u8::from(self.status.clone())],                    // STATUS | u8
            (self.description.len() as u32).to_be_bytes().to_vec(), // DESC_LEN | u32
            self.description.as_bytes().to_vec(),                   // DESCRIPTION | String
        ]
        .concat()
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let body = self.make_body();

        [self.make_header(&body).to_vec(), body].concat()
    }
}

pub struct ParserBin<R: Read> {
    reader: BufReader<R>,
}

impl<R: Read> ParserBin<R> {
    pub fn new(reader: R) -> Self {
        ParserBin {
            reader: BufReader::new(reader),
        }
    }
}

impl<R: Read> Parser for ParserBin<R> {
    type Reader = R;

    fn read_from(reader: Self::Reader) -> Result<Self, ParserError>
    where
        Self: Sized,
    {
        Ok(Self::new(reader))
    }

    fn write_record<W: Write>(record: &Record, writer: &mut W) -> Result<(), ParserError> {
        writer
            .write_all(&record.as_bytes())
            .map_err(|_| ParserError::NotImplemented)?;
        Ok(())
    }
}

impl<R: Read> Iterator for ParserBin<R> {
    type Item = Result<BinRecord, ParserError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut magic = [0u8; 4];

        match self.reader.read_exact(&mut magic) {
            Ok(_) => {
                if u32::from_be_bytes(magic) != MAGIC_HEADER {
                    return Some(Err(ParserError::RequiredFieldMissing));
                }
            }
            Err(e) => {
                return match e.kind() {
                    std::io::ErrorKind::UnexpectedEof => None,
                    _ => Some(Err(ParserError::NotImplemented)),
                };
            }
        }

        let mut record_size_u8 = [0u8; 4];
        if let Err(_) = self.reader.read_exact(&mut record_size_u8) {
            return Some(Err(ParserError::NotImplemented));
        };

        let record_size = u32::from_be_bytes(record_size_u8);
        let mut body = vec![0u8; record_size as usize];

        if let Err(_) = self.reader.read_exact(&mut body) {
            return Some(Err(ParserError::NotImplemented));
        }

        let (tx_id, body) = body.split_at(8);
        let (tx_type, body) = body.split_at(1);
        let (from_user_id, body) = body.split_at(8);
        let (to_user_id, body) = body.split_at(8);
        let (amount, body) = body.split_at(8);
        let (timestamp, body) = body.split_at(8);
        let (status, body) = body.split_at(1);
        let (desc_len, body) = body.split_at(4);
        let desc_len = u32::from_be_bytes(desc_len.try_into().expect("too many bytes"));
        let (description, _) = body.split_at(desc_len as usize);

        Some(Ok(BinRecord::new(
            u64::from_be_bytes(tx_id.try_into().expect("too many bytes")),
            TxType::try_from(tx_type[0]).expect("transaction type"),
            u64::from_be_bytes(from_user_id.try_into().expect("too many bytes")),
            u64::from_be_bytes(to_user_id.try_into().expect("too many bytes")),
            i64::from_be_bytes(amount.try_into().expect("too many bytes")),
            u64::from_be_bytes(timestamp.try_into().expect("too many bytes")),
            TxStatus::try_from(status[0]).expect("transaction status"),
            &String::from_utf8_lossy(description).to_string(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bin_format::{BinRecord, ParserBin};
    use crate::record::{TxStatus, TxType};
    use std::io::{BufReader, BufWriter, Cursor};
    use time::macros::datetime;

    #[test]
    fn test_read_from() {
        let data = vec![
            89, 80, 66, 78, 0, 0, 0, 63, 0, 3, 141, 126, 164, 198, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 127, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 100, 0, 0, 1, 124, 56,
            148, 250, 96, 1, 0, 0, 0, 17, 34, 82, 101, 99, 111, 114, 100, 32, 110, 117, 109, 98,
            101, 114, 32, 49, 34, 89, 80, 66, 78, 0, 0, 0, 63, 0, 3, 141, 126, 164, 198, 128, 1, 1,
            127, 255, 255, 255, 255, 255, 255, 255, 127, 255, 255, 255, 255, 255, 255, 255, 0, 0,
            0, 0, 0, 0, 0, 200, 0, 0, 1, 124, 56, 149, 228, 192, 2, 0, 0, 0, 17, 34, 82, 101, 99,
            111, 114, 100, 32, 110, 117, 109, 98, 101, 114, 32, 50, 34,
        ];

        let mut cursor = Cursor::new(&data[..]);
        let buffer = BufReader::new(&mut cursor);
        let mut parser = ParserBin::new(buffer);

        // let bools = parser
        //     .zip(1000000000000000u64..1000000000000010)
        //     .map(|(item, id)| item.map(|inner| inner.id == id))
        //     .collect::<Result<Vec<bool>, ParserError>>();
        //
        // println!("{:#?}", bools);

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
    fn test_write_record() {
        let mut buffer = Vec::new();

        let record = BinRecord::new(
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
                ParserBin::<std::io::Empty>::write_record(&record, &mut buff_writer),
                Ok(())
            )
        }

        assert_eq!(
            buffer.as_slice(),
            [
                89, 80, 66, 78, 0, 0, 0, 63, 0, 3, 141, 126, 164, 198, 128, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 127, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 100, 0, 0, 1,
                124, 56, 148, 250, 96, 1, 0, 0, 0, 17, 34, 82, 101, 99, 111, 114, 100, 32, 110,
                117, 109, 98, 101, 114, 32, 49, 34,
            ],
        )
    }

    #[test]
    fn test_record_bin_as_bytes() {
        let data = BinRecord::new(
            1000000000000000,
            TxType::Transfer,
            1001,
            455001,
            545,
            datetime!(2024-02-22 01:02:03.5 UTC).unix_timestamp() as u64,
            TxStatus::Success,
            "\"transfer for user 455001\"",
        );
        assert_eq!(
            data.as_bytes(),
            [
                89, 80, 66, 78, 0, 0, 0, 72, 0, 3, 141, 126, 164, 198, 128, 0, 1, 0, 0, 0, 0, 0, 0,
                3, 233, 0, 0, 0, 0, 0, 6, 241, 89, 0, 0, 0, 0, 0, 0, 2, 33, 0, 0, 0, 0, 101, 214,
                157, 11, 0, 0, 0, 0, 26, 34, 116, 114, 97, 110, 115, 102, 101, 114, 32, 102, 111,
                114, 32, 117, 115, 101, 114, 32, 52, 53, 53, 48, 48, 49, 34
            ]
        )
    }
}
