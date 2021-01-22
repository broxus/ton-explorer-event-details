#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

use std::collections::HashMap;
use std::io::Cursor;

use anyhow::{anyhow, Error, Result};
use ton_abi::{Contract, Function, Param, ParamType, TokenValue};
use ton_block::{
    Account, AccountState, CommonMsgInfo, Deserializable, ExternalInboundMessageHeader, InRefValue, Message,
    MsgAddressInt,
};
use ton_types::Cell;

use explorer_addon::contract;
use explorer_addon::tvm;

fn get_details_abi() -> Result<Function> {
    const TON_EVENT_ABI: &str = include_str!("TonEvent.abi.json");
    let abi = Contract::load(Cursor::new(TON_EVENT_ABI)).map_err(|_| anyhow!("Failed to load contract abi"))?;
    abi.function("getDetails")
        .map_err(|_| anyhow!("Failed to get getDetails function"))
        .map(|f| f.clone())
}

#[wasm_bindgen_test]
fn valid_details_abi() {
    assert!(get_details_abi().is_ok());
}

#[wasm_bindgen_test]
fn run_get_details() {
    const TVC: &str = include_str!("ton_event.tvc");
    let account = Account::construct_from_base64(TVC.trim()).unwrap();

    let state = match account.state().unwrap() {
        AccountState::AccountActive(state) => state,
        _ => panic!(""),
    };

    let data = state.data.clone().unwrap();
    let code = state.code.clone().unwrap();

    contract::get_details(code, data).unwrap();
}
