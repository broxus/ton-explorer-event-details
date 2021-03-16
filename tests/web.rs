#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

use std::collections::HashMap;
use std::io::Cursor;
use std::str::FromStr;

use num_bigint::BigUint;
use ton_abi::{Contract, Function, Param, ParamType, TokenValue};
use ton_block::{
    Account, AccountState, CommonMsgInfo, Deserializable, ExternalInboundMessageHeader, InRefValue, Message,
    MsgAddressInt,
};
use ton_types::Cell;
//
// #[wasm_bindgen_test]
// fn run_create_internal_body() {
//     const ABI: &str = include_str!("EthereumEvent.abi.json");
//     let abi = Contract::load(Cursor::new(ABI)).unwrap();
//     let function = abi.function("executeProxyCallback").unwrap();
//     let body = function.encode_input(&HashMap::new(), &[], true, None).unwrap();
//     let toc = ton_types::serialize_toc(&body.into_cell().unwrap()).unwrap();
//     panic!("Body: '{}'", base64::encode(&toc));
// }
//
// #[wasm_bindgen_test]
// fn run_swapback_body() {
//     const ABI: &str = include_str!("TokenEventProxy.abi.json");
//     let abi = Contract::load(Cursor::new(ABI)).unwrap();
//     let function = abi.function("transferMyTokensToEthereum").unwrap();
//     let body = function
//         .encode_input(
//             &HashMap::new(),
//             &[
//                 ton_abi::Token {
//                     value: ton_abi::TokenValue::Uint(ton_abi::Uint {
//                         number: BigUint::from(1000000u32),
//                         size: 128,
//                     }),
//                     name: "tokens".to_owned(),
//                 },
//                 ton_abi::Token {
//                     name: "ethereum_address".to_owned(),
//                     value: ton_abi::TokenValue::Bytes(hex::decode("1374EEE0dF363594B8E13A4Bb3F1e395e995893f").unwrap()),
//                 },
//             ],
//             true,
//             None,
//         )
//         .unwrap();
//     let toc = ton_types::serialize_toc(&body.into_cell().unwrap()).unwrap();
//     panic!("Body: '{}'", base64::encode(&toc));
// }
//
// #[wasm_bindgen_test]
// fn run_create_transfer_body() {
//     const ABI: &str = include_str!("TokenWallet.abi.json");
//     let abi = Contract::load(Cursor::new(ABI)).unwrap();
//     let function = abi.function("transfer").unwrap();
//     let body = ton_abi::TokenValue::pack_values_into_chain(
//         &[
//             ton_abi::Token {
//                 value: ton_abi::TokenValue::Uint(ton_abi::Uint::new(2021, 128)),
//                 name: "to".to_owned(),
//             },
//             ton_abi::Token {
//                 value: ton_abi::TokenValue::Int(ton_abi::Int::new(0, 8)),
//                 name: "wid".to_owned(),
//             },
//             ton_abi::Token {
//                 value: ton_abi::TokenValue::Uint(ton_abi::Uint::new(123456, 256)),
//                 name: "address".to_owned(),
//             },
//             ton_abi::Token {
//                 value: ton_abi::TokenValue::Uint(ton_abi::Uint::new(0, 256)),
//                 name: "key".to_owned(),
//             },
//         ],
//         Vec::new(),
//         2,
//     )
//     .unwrap();
//     let toc = ton_types::serialize_toc(&body.into_cell().unwrap()).unwrap();
//     panic!("Body: '{}'", base64::encode(&toc));
// }

#[wasm_bindgen_test]
fn decode_code() {
    const CODE: &str = "te6ccgEBAQEASwAAkmfOizcAvCXsqEs1frjhFnN1Sz6BMwvjMPctvcwpcXHffR2tvlAAAAAAAAAAAA3gtrOnZAAA5Wb757UCqPE6x6L7bvYW/DguWqY=";
    let toc = ton_types::deserialize_tree_of_cells(&mut std::io::Cursor::new(base64::decode(CODE).unwrap())).unwrap();
    panic!("{:#.1024}", toc);
}
