pub mod contract;
pub mod eth;
pub mod tvm;
mod utils;

use std::str::FromStr;

use num_bigint::BigUint;
use num_traits::ToPrimitive;
use ton_block::MsgAddressInt;
use wasm_bindgen::prelude::*;

use crate::utils::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(js_name = "getDetails")]
pub fn get_details(account_state: &str) -> Result<TonEventDetails, JsValue> {
    utils::set_panic_hook();
    let account_state = base64::decode(account_state).map_err(|_| "Failed to decode account state")?;
    let (code, data) = utils::decode_account_state(&account_state).handle_error()?;
    let details = contract::get_details(code, data).handle_error()?;
    convert_event_details(details).handle_error()
}

#[wasm_bindgen(js_name = "encodeEthAddress")]
pub fn encode_eth_address(address: &str) -> Result<String, JsValue> {
    utils::set_panic_hook();
    let address = ethabi::Address::from_str(address).map_err(|_| "Failed to decode proxy address")?;
    let data: ton_types::Cell = ton_abi::TokenValue::pack_values_into_chain(
        &[ton_abi::Token {
            name: String::default(),
            value: ton_abi::TokenValue::Uint(ton_abi::Uint {
                number: BigUint::from_bytes_be(&address.0),
                size: 160,
            }),
        }],
        Vec::new(),
        2,
    )
    .handle_error()?
    .into_cell()
    .handle_error()?;
    let data = ton_types::serialize_toc(&data).handle_error()?;
    Ok(base64::encode(&data))
}

#[wasm_bindgen(js_name = "encodePayload")]
pub fn encode_payload(event: &TonEventDetails, eth_abi: &str, proxy_address: &str) -> Result<String, JsValue> {
    utils::set_panic_hook();
    let proxy_address = ethabi::Address::from_str(proxy_address).map_err(|_| "Failed to decode proxy address")?;
    let payload = convert_eth_payload(event, proxy_address).handle_error()?;
    eth::encode_eth_payload(payload, eth_abi)
        .map(|payload| hex::encode(&payload))
        .handle_error()
}

fn convert_eth_payload(value: &TonEventDetails, proxy_address: ethabi::Address) -> Result<eth::EthPayload> {
    let event_transaction =
        hex::decode(&value.init_data.event_transaction).map_err(|_| "Failed to parse event transaction")?;
    let event_transaction_lt =
        u64::from_str(&value.init_data.event_transaction_lt).map_err(|_| "Failed to parse event transaction lt")?;

    let event_data = base64::decode(&value.init_data.event_data).map_err(|_| "Failed to parse Cell")?;
    let event_data = ton_types::deserialize_tree_of_cells(&mut std::io::Cursor::new(event_data))
        .map_err(|_| "Failed to parse Cell")?;

    let event_configuration = MsgAddressInt::from_str(&value.init_data.ton_event_configuration)
        .map_err(|_| "Failed to parse TON event configuration address")?;

    Ok(eth::EthPayload {
        event_transaction: event_transaction.into(),
        event_transaction_lt,
        event_timestamp: value.init_data.event_timestamp,
        event_index: value.init_data.event_index,
        event_data,
        event_configuration,
        required_confirmations: value.init_data.required_confirmations,
        required_rejections: value.init_data.required_rejections,
        proxy: proxy_address,
    })
}

#[wasm_bindgen]
pub struct TonEventDetails {
    init_data: TonEventInitData,
    status: EventStatus,
    confirmations: Vec<String>,
    rejections: Vec<String>,
    signatures: Vec<String>,
}

#[wasm_bindgen]
impl TonEventDetails {
    #[wasm_bindgen(getter = initData)]
    pub fn init_data(&self) -> TonEventInitData {
        self.init_data.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn status(&self) -> EventStatus {
        self.status
    }

    #[wasm_bindgen(getter)]
    pub fn confirmations(&self) -> js_sys::Array {
        self.confirmations.iter().map(JsValue::from).collect()
    }

    #[wasm_bindgen(getter)]
    pub fn rejections(&self) -> js_sys::Array {
        self.rejections.iter().map(JsValue::from).collect()
    }

    #[wasm_bindgen(getter)]
    pub fn signatures(&self) -> js_sys::Array {
        self.signatures.iter().map(JsValue::from).collect()
    }
}

fn convert_event_details(data: contract::TonEventDetails) -> Result<TonEventDetails> {
    Ok(TonEventDetails {
        init_data: convert_init_data(data.init_data)?,
        status: data.status.into(),
        confirmations: data.confirms.into_iter().map(|item| item.to_string()).collect(),
        rejections: data.rejections.into_iter().map(|item| item.to_string()).collect(),
        signatures: data.signatures.into_iter().map(|item| hex::encode(&item)).collect(),
    })
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct TonEventInitData {
    event_transaction: String,
    event_transaction_lt: String,
    event_timestamp: u32,
    event_index: u32,
    event_data: String,
    ton_event_configuration: String,
    required_confirmations: u16,
    required_rejections: u16,
    configuration_meta: String,
}

#[wasm_bindgen]
impl TonEventInitData {
    #[wasm_bindgen(getter = eventTransaction)]
    pub fn event_transaction(&self) -> String {
        self.event_transaction.clone()
    }

    #[wasm_bindgen(getter = eventTransactionLt)]
    pub fn event_transaction_lt(&self) -> String {
        self.event_transaction_lt.clone()
    }

    #[wasm_bindgen(getter = eventTimestamp)]
    pub fn event_timestamp(&self) -> u32 {
        self.event_timestamp
    }

    #[wasm_bindgen(getter = eventIndex)]
    pub fn event_index(&self) -> u32 {
        self.event_index
    }

    #[wasm_bindgen(getter = eventData)]
    pub fn event_data(&self) -> String {
        self.event_data.clone()
    }

    #[wasm_bindgen(getter = tonEventConfiguration)]
    pub fn ton_event_configuration(&self) -> String {
        self.ton_event_configuration.clone()
    }

    #[wasm_bindgen(getter = requiredConfirmations)]
    pub fn required_confirmations(&self) -> u16 {
        self.required_confirmations
    }

    #[wasm_bindgen(getter = requiredRejections)]
    pub fn required_rejections(&self) -> u16 {
        self.required_rejections
    }

    #[wasm_bindgen(getter = configurationMeta)]
    pub fn configuration_meta(&self) -> String {
        self.configuration_meta.clone()
    }
}

fn convert_init_data(data: contract::TonEventInitData) -> Result<TonEventInitData> {
    let event_data = match ton_types::serialize_toc(&data.event_data) {
        Ok(data) => base64::encode(&data),
        Err(_) => return Err("Failed to serialize Cell"),
    };

    let configuration_meta = match ton_types::serialize_toc(&data.configuration_meta) {
        Ok(data) => base64::encode(&data),
        Err(_) => return Err("Failed to serialize Cell"),
    };

    Ok(TonEventInitData {
        event_transaction: data.event_transaction.to_hex_string(),
        event_transaction_lt: data.event_transaction_lt.to_string(),
        event_timestamp: data.event_timestamp,
        event_index: data.event_index,
        event_data,
        ton_event_configuration: data.ton_event_configuration.to_string(),
        required_confirmations: data.required_confirmations.to_u16().ok_or("Invalid ABI")?,
        required_rejections: data.required_confirmations.to_u16().ok_or("Invalid ABI")?,
        configuration_meta,
    })
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum EventStatus {
    InProcess = "in_process",
    Confirmed = "confirmed",
    Rejected = "rejected",
}

impl From<contract::EventStatus> for EventStatus {
    fn from(status: contract::EventStatus) -> Self {
        match status {
            contract::EventStatus::InProcess => EventStatus::InProcess,
            contract::EventStatus::Confirmed => EventStatus::Confirmed,
            contract::EventStatus::Rejected => EventStatus::Rejected,
        }
    }
}

impl<T> HandleError for Result<T> {
    type Output = T;

    fn handle_error(self) -> Result<Self::Output, JsValue> {
        self.map_err(|e| js_sys::Error::new(&e.to_string()).into())
    }
}

impl<T> HandleError for ton_types::Result<T> {
    type Output = T;

    fn handle_error(self) -> Result<Self::Output, JsValue> {
        self.map_err(|e| js_sys::Error::new(&e.to_string()).into())
    }
}

trait HandleError {
    type Output;

    fn handle_error(self) -> Result<Self::Output, JsValue>;
}
