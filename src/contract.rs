use std::collections::HashMap;

use anyhow::{anyhow, Result};
use num_bigint::BigUint;
use serde::Deserialize;
use ton_abi::{Function, Param, ParamType, Token, TokenValue};
use ton_block::{CommonMsgInfo, ExternalInboundMessageHeader, Message, MsgAddress, MsgAddressInt};
use ton_types::{Cell, UInt256};

use crate::tvm;
use num_traits::ToPrimitive;

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
        .map_err(|_| anyhow!("Failed to encode input"))?;

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

        let body = message.body().ok_or_else(|| anyhow!("Out message must have a body"))?;

        if abi
            .is_my_output_message(body.clone(), false)
            .map_err(|_| anyhow!("Failed to check output message"))?
        {
            return abi
                .decode_output(body, false)
                .map_err(|_| anyhow!("Failed to decode output message"));
        }
    }

    Err(anyhow!("No output messages found"))
}

pub struct TonEventDetails {
    pub init_data: TonEventInitData,
    pub status: EventStatus,
    pub confirms: Vec<MsgAddressInt>,
    pub rejections: Vec<MsgAddressInt>,
    pub signatures: Vec<Vec<u8>>,
}

impl TryParse<TonEventDetails> for Vec<Token> {
    fn try_parse(self) -> ParseResult<TonEventDetails> {
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
    pub event_index: u32,
    pub event_data: Cell,

    pub ton_event_configuration: MsgAddressInt,
    pub required_confirmations: BigUint,
    pub required_rejections: BigUint,
}

impl TryParse<TonEventInitData> for TokenValue {
    fn try_parse(self) -> ParseResult<TonEventInitData> {
        let mut tuple = match self {
            TokenValue::Tuple(tuple) => tuple.into_iter(),
            _ => return Err(InvalidAbiError),
        };

        Ok(TonEventInitData {
            event_transaction: tuple.next().try_parse()?,
            event_transaction_lt: tuple.next().try_parse()?,
            event_index: tuple.next().try_parse()?,
            event_data: tuple.next().try_parse()?,
            ton_event_configuration: tuple.next().try_parse()?,
            required_confirmations: tuple.next().try_parse()?,
            required_rejections: tuple.next().try_parse()?,
        })
    }
}

pub enum EventStatus {
    InProcess,
    Confirmed,
    Rejected,
}

impl TryParse<EventStatus> for TokenValue {
    fn try_parse(self) -> ParseResult<EventStatus> {
        match self {
            TokenValue::Uint(value) => match value.number.to_u8() {
                Some(0) => Ok(EventStatus::InProcess),
                Some(1) => Ok(EventStatus::Confirmed),
                Some(2) => Ok(EventStatus::Rejected),
                _ => Err(InvalidAbiError),
            },
            _ => Err(InvalidAbiError),
        }
    }
}

impl TryParse<Cell> for TokenValue {
    fn try_parse(self) -> ParseResult<Cell> {
        match self {
            TokenValue::Cell(cell) => Ok(cell),
            _ => Err(InvalidAbiError),
        }
    }
}

impl TryParse<Vec<MsgAddressInt>> for TokenValue {
    fn try_parse(self) -> ParseResult<Vec<MsgAddressInt>> {
        match self {
            TokenValue::Array(tuple) => tuple.into_iter().map(TryParse::<MsgAddressInt>::try_parse).collect(),
            _ => Err(InvalidAbiError),
        }
    }
}

impl TryParse<MsgAddressInt> for TokenValue {
    fn try_parse(self) -> ParseResult<MsgAddressInt> {
        match self {
            TokenValue::Address(address) => match address {
                MsgAddress::AddrStd(address) => Ok(MsgAddressInt::AddrStd(address)),
                MsgAddress::AddrVar(address) => Ok(MsgAddressInt::AddrVar(address)),
                _ => Err(InvalidAbiError),
            },
            _ => Err(InvalidAbiError),
        }
    }
}

impl TryParse<BigUint> for TokenValue {
    fn try_parse(self) -> ParseResult<BigUint> {
        match self {
            TokenValue::Uint(data) => Ok(data.number),
            _ => Err(InvalidAbiError),
        }
    }
}

impl TryParse<u64> for TokenValue {
    fn try_parse(self) -> ParseResult<u64> {
        match self {
            TokenValue::Uint(data) => data.number.to_u64().ok_or(InvalidAbiError),

            _ => Err(InvalidAbiError),
        }
    }
}

impl TryParse<u32> for TokenValue {
    fn try_parse(self) -> ParseResult<u32> {
        match self {
            TokenValue::Uint(data) => data.number.to_u32().ok_or(InvalidAbiError),

            _ => Err(InvalidAbiError),
        }
    }
}

impl TryParse<Vec<Vec<u8>>> for TokenValue {
    fn try_parse(self) -> ParseResult<Vec<Vec<u8>>> {
        match self {
            TokenValue::Array(tokens) => tokens.into_iter().map(TryParse::<Vec<u8>>::try_parse).collect(),
            _ => Err(InvalidAbiError),
        }
    }
}

impl TryParse<Vec<u8>> for TokenValue {
    fn try_parse(self) -> ParseResult<Vec<u8>> {
        match self {
            TokenValue::Bytes(tokens) => Ok(tokens),
            _ => Err(InvalidAbiError),
        }
    }
}

impl TryParse<u8> for TokenValue {
    fn try_parse(self) -> ParseResult<u8> {
        match self {
            TokenValue::Uint(data) => data.number.to_u8().ok_or(InvalidAbiError),

            _ => Err(InvalidAbiError),
        }
    }
}

impl TryParse<UInt256> for TokenValue {
    fn try_parse(self) -> ParseResult<UInt256> {
        match self {
            TokenValue::Uint(data) => Ok(data.number.to_bytes_be().into()),
            _ => Err(InvalidAbiError),
        }
    }
}

impl<T> TryParse<T> for Option<Token>
where
    TokenValue: TryParse<T>,
{
    fn try_parse(self) -> ParseResult<T> {
        self.map(|data| data.value).try_parse()
    }
}

impl<T> TryParse<T> for Option<TokenValue>
where
    TokenValue: TryParse<T>,
{
    fn try_parse(self) -> ParseResult<T> {
        match self {
            Some(data) => data.try_parse(),
            None => Err(InvalidAbiError),
        }
    }
}

impl<T> TryParse<T> for Token
where
    TokenValue: TryParse<T>,
{
    fn try_parse(self) -> ParseResult<T> {
        self.value.try_parse()
    }
}

trait TryParse<T>: Sized {
    fn try_parse(self) -> ParseResult<T>;
}

type ParseResult<T> = Result<T, InvalidAbiError>;

#[derive(Debug, thiserror::Error)]
#[error("Invalid ABI")]
struct InvalidAbiError;

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
    {"components":[{"name":"eventTransaction","type":"uint256"},{"name":"eventTransactionLt","type":"uint64"},{"name":"eventIndex","type":"uint32"},{"name":"eventData","type":"cell"},{"name":"tonEventConfiguration","type":"address"},{"name":"requiredConfirmations","type":"uint256"},{"name":"requiredRejects","type":"uint256"}],"name":"_initData","type":"tuple"},
    {"name":"_status","type":"uint8"},
    {"name":"_confirmRelays","type":"address[]"},
    {"name":"_rejectRelays","type":"address[]"},
    {"name":"_eventDataSignatures","type":"bytes[]"}
  ]
}"#;
