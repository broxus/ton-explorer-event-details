use std::sync::Arc;

use ton_block::{
    CommonMsgInfo, CurrencyCollection, Deserializable, Message, MsgAddressInt, OutAction, OutActions, Serializable,
};
use ton_types::{Cell, SliceData};
use ton_vm::executor::gas::gas_state::Gas;
use ton_vm::stack::integer::IntegerData;
use ton_vm::stack::{savelist::SaveList, Stack, StackItem};

use crate::utils::Result;

const ONE_TON: u64 = 1_000_000_000;
const BALANCE: u64 = 100 * ONE_TON;

pub fn call_msg(
    addr: MsgAddressInt,
    utime: u32,
    lt: u64,
    code: Cell,
    data: Cell,
    msg: &Message,
) -> Result<Vec<Message>> {
    let msg_cell = msg.write_to_new_cell().map_err(|_| "Failed to serialize message")?;

    let mut stack = Stack::new();
    let function_selector = match msg.header() {
        CommonMsgInfo::IntMsgInfo(_) => ton_vm::int!(0),
        CommonMsgInfo::ExtInMsgInfo(_) => ton_vm::int!(-1),
        CommonMsgInfo::ExtOutMsgInfo(_) => return Err("Invalid message type"),
    };

    stack
        .push(ton_vm::int!(BALANCE)) // token balance of contract
        .push(ton_vm::int!(0)) // token balance of msg
        .push(StackItem::Cell(msg_cell.into())) // message
        .push(StackItem::Slice(msg.body().unwrap_or_default())) // message body
        .push(function_selector); // function selector

    let engine = call(utime, lt, addr, code, data, stack)?;

    // process out actions to get out messages
    let actions_cell = engine
        .get_actions()
        .as_cell()
        .map_err(|_| "Can not get actions")?
        .clone();

    let mut actions = OutActions::construct_from_cell(actions_cell).map_err(|_| "Failed to parse actions")?;

    let mut msgs = Vec::new();
    for (_, action) in actions.iter_mut().enumerate() {
        if let OutAction::SendMsg { out_msg, .. } = std::mem::replace(action, OutAction::None) {
            msgs.push(out_msg);
        }
    }

    msgs.reverse();
    Ok(msgs)
}

pub fn call(
    block_unix_time: u32,
    block_lt: u64,
    addr: MsgAddressInt,
    code: Cell,
    data: Cell,
    stack: Stack,
) -> Result<ton_vm::executor::Engine> {
    let mut ctrls = SaveList::new();
    ctrls
        .put(4, &mut StackItem::Cell(data))
        .map_err(|_| "Failed to put data to registers")?;

    let sci = build_contract_info(
        &addr,
        &CurrencyCollection {
            grams: BALANCE.into(),
            other: Default::default(),
        },
        block_unix_time,
        block_lt,
        block_lt,
    );

    ctrls
        .put(7, &mut sci.into_temp_data())
        .map_err(|_| "Failed to put SCI to registers")?;

    let gas_limit = 1_000_000_000;
    let gas = Gas::new(gas_limit, 0, gas_limit, 10);

    let mut engine = ton_vm::executor::Engine::new().setup(SliceData::from(code), Some(ctrls), Some(stack), Some(gas));

    let _ = engine.execute().map_err(|_| "TVM execution failed")?;

    Ok(engine)
}

fn build_contract_info(
    address: &MsgAddressInt,
    balance: &CurrencyCollection,
    block_unix_time: u32,
    block_lt: u64,
    tr_lt: u64,
) -> ton_vm::SmartContractInfo {
    let mut info = ton_vm::SmartContractInfo::with_myself(address.serialize().unwrap_or_default().into());
    *info.block_lt_mut() = block_lt;
    *info.trans_lt_mut() = tr_lt;
    *info.unix_time_mut() = block_unix_time;
    *info.balance_remaining_grams_mut() = balance.grams.0;
    *info.balance_remaining_other_mut() = balance.other_as_hashmap();

    info
}
