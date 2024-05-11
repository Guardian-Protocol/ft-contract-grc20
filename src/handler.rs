use gstd::msg;
use io::{FTAction, FTEvent};

use crate::{contract::FungibleToken, FUNGIBLE_TOKEN};

#[no_mangle]
extern "C" fn handle() {
    let action: FTAction = msg::load().expect("Could not load the action");

    let ft: &mut FungibleToken = unsafe {
        FUNGIBLE_TOKEN
            .as_mut()
            .expect("The contract is not initialized")
    };

    let reply = match action {
        FTAction::TransferToUsers { amount, to_users } => ft.transfer_to_users(amount, to_users),
        FTAction::Mint { amount, to } => ft.mint(amount, to),
        FTAction::Burn { amount } => ft.burn(amount),
        FTAction::Transfer { 
            tx_id, 
            from,
            to,
            amount 
        } => ft.transfer(tx_id, &from, &to, amount),
        FTAction::Approve { tx_id, to, amount } => ft.approve(tx_id, &to, amount),
        FTAction::BalanceOf(account) => {
            let balance = ft.balances.get(&account).unwrap_or(&0);
            Ok(FTEvent::Balance(*balance))
        },
        FTAction::AddAdmin { admin_id } => ft.add_admin(&admin_id),
        FTAction::DeleteAdmin { admin_id } => ft.delete_admin(&admin_id),
    };

    msg::reply(reply, 0).expect("Error in sending a reply");
}