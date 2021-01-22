use ton_block::{Account, AccountState, Deserializable};
use ton_types::Cell;

pub type Result<T, E = &'static str> = core::result::Result<T, E>;

pub fn decode_account_state(account_state: &[u8]) -> Result<(Cell, Cell)> {
    let account = Account::construct_from_bytes(account_state).map_err(|_| "Failed to decode account state")?;

    let state = match account.state() {
        Some(AccountState::AccountActive(state)) => state,
        _ => return Err("Account is not active"),
    };

    match (state.code.clone(), state.data.clone()) {
        (Some(code), Some(data)) => Ok((code, data)),
        (None, _) => Err("Account doesn't have code"),
        (_, None) => Err("Account doesn't have data"),
    }
}

pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}
