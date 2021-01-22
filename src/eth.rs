use anyhow::{anyhow, Result};
use ethabi::{Token as EthTokenValue, Token};
use num_bigint::{BigInt, BigUint};
use serde::Deserialize;
use ton_abi::{Event as AbiEvent, Token as TonToken, TokenValue as TonTokenValue};
use ton_types::UInt256;

use crate::contract::TonEventDetails;

pub fn decode_event_data(event: TonEventDetails, event_abi: &str) -> Result<Vec<u8>> {
    let event_abi = serde_json::from_str::<SwapBackEventAbi>(event_abi)?;
    let mut abi = AbiEvent {
        abi_version: 2,
        name: event_abi.name,
        inputs: event_abi.inputs,
        id: 0,
    };
    abi.id = if let Some(id) = event_abi.id {
        id
    } else {
        abi.get_function_id() & 0x7FFFFFFF
    };

    let decoded = abi
        .decode_input(event.init_data.event_data.into())
        .map_err(|_| anyhow!("Failed to decode TON event data"))?;

    let event_data = map_event_data(decoded)?;

    let tuple = EthTokenValue::Tuple(vec![
        event.init_data.event_transaction.pack(),
        event.init_data.event_transaction_lt.pack(),
        event.init_data.event_index.pack(),
        event_data.pack(),
        (event.init_data.ton_event_configuration.workchain_id() as i8).pack(),
        UInt256::from(event.init_data.ton_event_configuration.address().get_bytestring(0)).pack(),
        event.init_data.required_confirmations.pack(),
        event.init_data.required_rejections.pack(),
    ]);

    Ok(ethabi::encode(&[tuple]).to_vec())
}

pub fn map_event_data(tokens: Vec<TonToken>) -> Result<Vec<u8>> {
    let tokens: Vec<_> = tokens
        .into_iter()
        .map(|token| token.value)
        .map(map_ton_to_eth)
        .collect::<Result<_, _>>()?;

    Ok(ethabi::encode(&tokens).to_vec())
}

fn map_ton_to_eth(token: TonTokenValue) -> Result<EthTokenValue> {
    Ok(match token {
        TonTokenValue::FixedBytes(bytes) => EthTokenValue::FixedBytes(bytes),
        TonTokenValue::Bytes(bytes) => EthTokenValue::Bytes(bytes),
        TonTokenValue::Uint(a) => {
            let bytes = a.number.to_bytes_le();
            EthTokenValue::Uint(ethabi::Uint::from_little_endian(&bytes))
        }
        TonTokenValue::Int(a) => {
            let mut bytes = a.number.to_signed_bytes_le();
            let sign = bytes.last().map(|first| (first >> 7) * 255).unwrap_or_default();
            bytes.resize(32, sign);
            //fixme check it
            EthTokenValue::Int(ethabi::Int::from_little_endian(&bytes))
        }
        TonTokenValue::Bool(a) => EthTokenValue::Bool(a),
        TonTokenValue::FixedArray(tokens) => {
            EthTokenValue::FixedArray(tokens.into_iter().map(map_ton_to_eth).collect::<Result<_, _>>()?)
        }
        TonTokenValue::Array(tokens) => {
            EthTokenValue::Array(tokens.into_iter().map(map_ton_to_eth).collect::<Result<_, _>>()?)
        }
        TonTokenValue::Tuple(tokens) => EthTokenValue::Tuple(
            tokens
                .into_iter()
                .map(|ton| map_ton_to_eth(ton.value))
                .collect::<Result<_, _>>()?,
        ),
        _ => return Err(anyhow!("Unsupported type")),
    })
}

#[derive(Debug, Clone, Deserialize)]
struct SwapBackEventAbi {
    name: String,

    #[serde(default)]
    inputs: Vec<ton_abi::Param>,

    #[serde(default)]
    #[serde(deserialize_with = "ton_abi::contract::deserialize_opt_u32_from_string")]
    id: Option<u32>,
}

impl Pack for UInt256 {
    fn pack(self) -> EthTokenValue {
        EthTokenValue::Uint(ethereum_types::U256::from_big_endian(self.as_slice()))
    }
}

impl Pack for BigUint {
    fn pack(self) -> Token {
        let bytes = self.to_bytes_le();
        EthTokenValue::Uint(ethabi::Uint::from_little_endian(&bytes))
    }
}

impl Pack for BigInt {
    fn pack(self) -> Token {
        let mut bytes = self.to_signed_bytes_le();
        let sign = bytes.last().map(|first| (first >> 7) * 255).unwrap_or_default();
        bytes.resize(32, sign);
        EthTokenValue::Int(ethabi::Int::from_little_endian(&bytes))
    }
}

impl Pack for u64 {
    fn pack(self) -> Token {
        BigUint::from(self).pack()
    }
}

impl Pack for u32 {
    fn pack(self) -> Token {
        BigUint::from(self).pack()
    }
}

impl Pack for i8 {
    fn pack(self) -> Token {
        BigInt::from(self).pack()
    }
}

impl Pack for Vec<u8> {
    fn pack(self) -> Token {
        EthTokenValue::Bytes(self)
    }
}

pub trait Pack {
    fn pack(self) -> EthTokenValue;
}
