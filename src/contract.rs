use std::collections::HashMap;

use ethabi::Address;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use serde::Deserialize;
use ton_abi::{Function, Param, ParamType, Token, TokenValue};
use ton_block::{CommonMsgInfo, ExternalInboundMessageHeader, Message, MsgAddress, MsgAddressInt};
use ton_types::{Cell, UInt256};

use crate::tvm;
use crate::utils::Result;

pub fn get_details(code: Cell, data: Cell) -> Result<TonEventDetails> {
    let tokens = execute_message(code, data)?;
    let details = tokens.try_parse()?;
    Ok(details)
}

fn execute_message(code: Cell, data: Cell) -> Result<Vec<Token>> {
    let mut header = HashMap::new();
    header.insert("time".to_owned(), TokenValue::Time(1));
    header.insert("expire".to_owned(), TokenValue::Expire(1000));

    let abi = abi_get_details();

    let message = abi
        .encode_input(&header, &[], false, None)
        .map_err(|_| "Failed to encode input")?;

    let addr = MsgAddressInt::default();
    let mut msg = Message::with_ext_in_header(ExternalInboundMessageHeader {
        dst: addr.clone(),
        ..Default::default()
    });
    msg.set_body(message.into());

    let messages = tvm::call_msg(addr, 1, 1, code, data, &msg)?;

    for message in messages.into_iter() {
        if !matches!(message.header(), CommonMsgInfo::ExtOutMsgInfo(_)) {
            continue;
        }

        let body = message.body().ok_or("Out message must have a body")?;

        if abi
            .is_my_output_message(body.clone(), false)
            .map_err(|_| "Failed to check output message")?
        {
            return abi
                .decode_output(body, false)
                .map_err(|_| "Failed to decode output message");
        }
    }

    Err("No output messages found")
}

pub struct TonEventDetails {
    pub init_data: TonEventInitData,
    pub status: EventStatus,
    pub confirms: Vec<MsgAddressInt>,
    pub rejections: Vec<MsgAddressInt>,
    pub signatures: Vec<Vec<u8>>,
}

impl TryParse<TonEventDetails> for Vec<Token> {
    fn try_parse(self) -> Result<TonEventDetails> {
        let mut tuple = self.into_iter();

        Ok(TonEventDetails {
            init_data: tuple.next().try_parse()?,
            status: tuple.next().try_parse()?,
            confirms: tuple.next().try_parse()?,
            rejections: tuple.next().try_parse()?,
            signatures: tuple.next().try_parse()?,
        })
    }
}

pub struct TonEventInitData {
    pub event_transaction: UInt256,
    pub event_transaction_lt: u64,
    pub event_timestamp: u32,
    pub event_index: u32,
    pub event_data: Cell,

    pub ton_event_configuration: MsgAddressInt,
    pub required_confirmations: u16,
    pub required_rejections: u16,

    pub configuration_meta: Cell,
}

impl TryParse<TonEventInitData> for TokenValue {
    fn try_parse(self) -> Result<TonEventInitData> {
        let mut tuple = match self {
            TokenValue::Tuple(tuple) => tuple.into_iter(),
            _ => return Err(INVALID_ABI),
        };

        Ok(TonEventInitData {
            event_transaction: tuple.next().try_parse()?,
            event_transaction_lt: tuple.next().try_parse()?,
            event_timestamp: tuple.next().try_parse()?,
            event_index: tuple.next().try_parse()?,
            event_data: tuple.next().try_parse()?,
            ton_event_configuration: tuple.next().try_parse()?,
            required_confirmations: tuple.next().try_parse()?,
            required_rejections: tuple.next().try_parse()?,
            configuration_meta: tuple.next().try_parse()?,
        })
    }
}

pub enum EventStatus {
    InProcess,
    Confirmed,
    Rejected,
}

impl TryParse<EventStatus> for TokenValue {
    fn try_parse(self) -> Result<EventStatus> {
        match self {
            TokenValue::Uint(value) => match value.number.to_u8() {
                Some(0) => Ok(EventStatus::InProcess),
                Some(1) => Ok(EventStatus::Confirmed),
                Some(2) => Ok(EventStatus::Rejected),
                _ => Err(INVALID_ABI),
            },
            _ => Err(INVALID_ABI),
        }
    }
}

impl TryParse<Cell> for TokenValue {
    fn try_parse(self) -> Result<Cell> {
        match self {
            TokenValue::Cell(cell) => Ok(cell),
            _ => Err(INVALID_ABI),
        }
    }
}

impl TryParse<Vec<MsgAddressInt>> for TokenValue {
    fn try_parse(self) -> Result<Vec<MsgAddressInt>> {
        match self {
            TokenValue::Array(tuple) => tuple.into_iter().map(TryParse::<MsgAddressInt>::try_parse).collect(),
            _ => Err(INVALID_ABI),
        }
    }
}

impl TryParse<MsgAddressInt> for TokenValue {
    fn try_parse(self) -> Result<MsgAddressInt> {
        match self {
            TokenValue::Address(address) => match address {
                MsgAddress::AddrStd(address) => Ok(MsgAddressInt::AddrStd(address)),
                MsgAddress::AddrVar(address) => Ok(MsgAddressInt::AddrVar(address)),
                _ => Err(INVALID_ABI),
            },
            _ => Err(INVALID_ABI),
        }
    }
}

impl TryParse<BigUint> for TokenValue {
    fn try_parse(self) -> Result<BigUint> {
        match self {
            TokenValue::Uint(data) => Ok(data.number),
            _ => Err(INVALID_ABI),
        }
    }
}

impl TryParse<u64> for TokenValue {
    fn try_parse(self) -> Result<u64> {
        match self {
            TokenValue::Uint(data) => data.number.to_u64().ok_or(INVALID_ABI),
            _ => Err(INVALID_ABI),
        }
    }
}

impl TryParse<u32> for TokenValue {
    fn try_parse(self) -> Result<u32> {
        match self {
            TokenValue::Uint(data) => data.number.to_u32().ok_or(INVALID_ABI),
            _ => Err(INVALID_ABI),
        }
    }
}

impl TryParse<u16> for TokenValue {
    fn try_parse(self) -> Result<u16> {
        match self {
            TokenValue::Uint(data) => data.number.to_u16().ok_or(INVALID_ABI),
            _ => Err(INVALID_ABI),
        }
    }
}

impl TryParse<Vec<Vec<u8>>> for TokenValue {
    fn try_parse(self) -> Result<Vec<Vec<u8>>> {
        match self {
            TokenValue::Array(tokens) => tokens.into_iter().map(TryParse::<Vec<u8>>::try_parse).collect(),
            _ => Err(INVALID_ABI),
        }
    }
}

impl TryParse<Vec<u8>> for TokenValue {
    fn try_parse(self) -> Result<Vec<u8>> {
        match self {
            TokenValue::Bytes(tokens) => Ok(tokens),
            _ => Err(INVALID_ABI),
        }
    }
}

impl TryParse<u8> for TokenValue {
    fn try_parse(self) -> Result<u8> {
        match self {
            TokenValue::Uint(data) => data.number.to_u8().ok_or(INVALID_ABI),

            _ => Err(INVALID_ABI),
        }
    }
}

impl TryParse<UInt256> for TokenValue {
    fn try_parse(self) -> Result<UInt256> {
        match self {
            TokenValue::Uint(data) => Ok(data.number.to_bytes_be().into()),
            _ => Err(INVALID_ABI),
        }
    }
}

impl TryParse<ethabi::Address> for TokenValue {
    fn try_parse(self) -> Result<Address> {
        match self {
            TokenValue::Uint(value) => {
                let mut address = ethereum_types::Address::default();
                let bytes = value.number.to_bytes_be();

                const ADDRESS_SIZE: usize = 20;

                // copy min(N,20) bytes into last min(N,20) elements of address

                let size = bytes.len();
                let src_offset = size - size.min(ADDRESS_SIZE);
                let dest_offset = ADDRESS_SIZE - size.min(ADDRESS_SIZE);
                address.0[dest_offset..ADDRESS_SIZE].copy_from_slice(&bytes[src_offset..size]);

                Ok(address)
            }
            _ => Err(INVALID_ABI),
        }
    }
}

impl<T> TryParse<T> for Option<Token>
where
    TokenValue: TryParse<T>,
{
    fn try_parse(self) -> Result<T> {
        self.map(|data| data.value).try_parse()
    }
}

impl<T> TryParse<T> for Option<TokenValue>
where
    TokenValue: TryParse<T>,
{
    fn try_parse(self) -> Result<T> {
        match self {
            Some(data) => data.try_parse(),
            None => Err(INVALID_ABI),
        }
    }
}

impl<T> TryParse<T> for Token
where
    TokenValue: TryParse<T>,
{
    fn try_parse(self) -> Result<T> {
        self.value.try_parse()
    }
}

trait TryParse<T>: Sized {
    fn try_parse(self) -> Result<T>;
}

const INVALID_ABI: &str = "Invalid ABI";

pub fn abi_get_details() -> Function {
    let abi = serde_json::from_str::<GetDetailsAbiFunction>(ABI).unwrap();
    let mut abi = Function {
        abi_version: 2,
        name: abi.name,
        header: vec![
            Param {
                name: "time".to_string(),
                kind: ParamType::Time,
            },
            Param {
                name: "expire".to_string(),
                kind: ParamType::Expire,
            },
        ],
        inputs: Vec::new(),
        outputs: abi.outputs,
        input_id: 0,
        output_id: 0,
    };
    let id = abi.get_function_id();
    abi.input_id = id & 0x7FFFFFFF;
    abi.output_id = id | 0x80000000;
    abi
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct GetDetailsAbiFunction {
    pub name: String,
    pub outputs: Vec<Param>,
}

const ABI: &str = r#"{
  "name": "getDetails",
  "outputs": [
    {"components":[{"name":"eventTransaction","type":"uint256"},{"name":"eventTransactionLt","type":"uint64"},{"name":"eventTimestamp","type":"uint32"},{"name":"eventIndex","type":"uint32"},{"name":"eventData","type":"cell"},{"name":"tonEventConfiguration","type":"address"},{"name":"requiredConfirmations","type":"uint16"},{"name":"requiredRejects","type":"uint16"},{"name":"configurationMeta","type":"cell"}],"name":"_initData","type":"tuple"},
    {"name":"_status","type":"uint8"},
    {"name":"_confirmRelays","type":"address[]"},
    {"name":"_rejectRelays","type":"address[]"},
    {"name":"_eventDataSignatures","type":"bytes[]"}
  ]
}"#;
