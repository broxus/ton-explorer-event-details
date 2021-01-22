pub mod contract;
pub mod eth;
pub mod tvm;
mod utils;

use std::convert::{TryFrom, TryInto};

use anyhow::{anyhow, Error, Result};
use num_traits::ToPrimitive;
use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn run() {
    utils::set_panic_hook();
}

#[wasm_bindgen]
pub fn get_details(account_state: &[u8]) -> Result<TonEventDetails, JsValue> {
    let (code, data) = utils::decode_account_state(account_state).handle_error()?;
    let details = contract::get_details(code, data).handle_error()?;
    details.try_into().handle_error()
}

#[wasm_bindgen]
pub struct TonEventDetails {
    init_data: TonEventInitData,
    status: EventStatus,
    confirms: Vec<String>,
    rejections: Vec<String>,
    signatures: Vec<Vec<u8>>,
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
        self.signatures
            .iter()
            .map(|item| js_sys::Uint8Array::from(item.as_slice()))
            .collect()
    }
}

impl TryFrom<contract::TonEventDetails> for TonEventDetails {
    type Error = Error;

    fn try_from(value: contract::TonEventDetails) -> Result<Self> {
        Ok(Self {
            init_data: value.init_data.try_into()?,
            status: value.status.into(),
            confirms: value.confirms.into_iter().map(|item| item.to_string()).collect(),
            rejections: value.rejections.into_iter().map(|item| item.to_string()).collect(),
            signatures: value.signatures,
        })
    }
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

impl TryFrom<contract::TonEventInitData> for TonEventInitData {
    type Error = Error;

    fn try_from(data: contract::TonEventInitData) -> Result<Self> {
        let event_data = match ton_types::serialize_toc(&data.event_data) {
            Ok(data) => base64::encode(&data),
            Err(_) => return Err(anyhow!("Failed to serialize Cell")),
        };

        Ok(Self {
            event_transaction: data.event_transaction.to_hex_string(),
            event_transaction_lt: data.event_transaction_lt.to_string(),
            event_index: data.event_index,
            event_data,
            ton_event_configuration: data.ton_event_configuration.to_string(),
            required_confirmations: data
                .required_confirmations
                .to_u16()
                .ok_or_else(|| anyhow!("Invalid ABI"))?,
            required_rejections: data
                .required_confirmations
                .to_u16()
                .ok_or_else(|| anyhow!("Invalid ABI"))?,
        })
    }
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
