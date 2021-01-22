pub mod contract;
pub mod eth;
pub mod tvm;
mod utils;

use std::str::FromStr;

use num_traits::ToPrimitive;
use ton_block::MsgAddressInt;
use wasm_bindgen::prelude::*;

use crate::utils::Result;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn run() {
    utils::set_panic_hook();
}
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn get_details(account_state: &[u8]) -> Result<TonEventDetails, JsValue> {
    let (code, data) = utils::decode_account_state(account_state).handle_error()?;
    let details = contract::get_details(code, data).handle_error()?;
    convert_event_details(details).handle_error()
}

#[wasm_bindgen]
pub fn encode_eth_payload(event: &TonEventDetails, eth_abi: &str) -> Result<String, JsValue> {
    let payload = convert_eth_payload(event).handle_error()?;
    eth::encode_eth_payload(payload, eth_abi)
        .map(|payload| base64::encode(&payload))
        .handle_error()
}

fn convert_eth_payload(value: &TonEventDetails) -> Result<eth::EthPayload> {
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
        event_index: value.init_data.event_index,
        event_data,
        event_configuration,
        required_confirmations: value.init_data.required_confirmations,
        required_rejections: value.init_data.required_rejections,
    })
}

#[wasm_bindgen]
pub struct TonEventDetails {
    init_data: TonEventInitData,
    status: EventStatus,
    confirms: Vec<String>,
    rejections: Vec<String>,
    signatures: Vec<String>,
}

#[wasm_bindgen]
impl TonEventDetails {
    #[wasm_bindgen(getter)]
    pub fn init_data(&self) -> TonEventInitData {
        self.init_data.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn status(&self) -> EventStatus {
        self.status
    }

    #[wasm_bindgen(getter)]
    pub fn confirms(&self) -> js_sys::Array {
        self.confirms.iter().map(JsValue::from).collect()
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
        confirms: data.confirms.into_iter().map(|item| item.to_string()).collect(),
        rejections: data.rejections.into_iter().map(|item| item.to_string()).collect(),
        signatures: data.signatures.into_iter().map(|item| base64::encode(&item)).collect(),
    })
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct TonEventInitData {
    event_transaction: String,
    event_transaction_lt: String,
    event_index: u32,
    event_data: String,
    ton_event_configuration: String,
    required_confirmations: u16,
    required_rejections: u16,
}

#[wasm_bindgen]
impl TonEventInitData {
    #[wasm_bindgen(getter)]
    pub fn event_transaction(&self) -> String {
        self.event_transaction.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn event_transaction_lt(&self) -> String {
        self.event_transaction_lt.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn event_index(&self) -> u32 {
        self.event_index
    }

    #[wasm_bindgen(getter)]
    pub fn event_data(&self) -> String {
        self.event_data.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn ton_event_configuration(&self) -> String {
        self.ton_event_configuration.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn required_confirmations(&self) -> u16 {
        self.required_confirmations
    }

    #[wasm_bindgen(getter)]
    pub fn required_rejections(&self) -> u16 {
        self.required_rejections
    }
}

fn convert_init_data(data: contract::TonEventInitData) -> Result<TonEventInitData> {
    let event_data = match ton_types::serialize_toc(&data.event_data) {
        Ok(data) => base64::encode(&data),
        Err(_) => return Err("Failed to serialize Cell"),
    };

    Ok(TonEventInitData {
        event_transaction: data.event_transaction.to_hex_string(),
        event_transaction_lt: data.event_transaction_lt.to_string(),
        event_index: data.event_index,
        event_data,
        ton_event_configuration: data.ton_event_configuration.to_string(),
        required_confirmations: data.required_confirmations.to_u16().ok_or("Invalid ABI")?,
        required_rejections: data.required_confirmations.to_u16().ok_or("Invalid ABI")?,
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

trait HandleError {
    type Output;

    fn handle_error(self) -> Result<Self::Output, JsValue>;
}
