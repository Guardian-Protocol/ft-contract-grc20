#![no_std]
use contract::FungibleToken;
use gstd::{collections::HashMap, msg, vec, ActorId, Vec};
use io::{FTError, FTEvent, FTQuery, FTQueryReply, InitFt, TxId};

pub mod contract;
pub mod handler;

pub const ZERO_ID: ActorId = ActorId::new([0u8; 32]);

static mut FUNGIBLE_TOKEN: Option<FungibleToken> = None;

#[no_mangle]
extern "C" fn init() {
    let init_config: InitFt = msg::load().expect("Unable to decode InitConfig");

    if init_config.description.chars().count() > 500 {
        msg::reply(FTError::DescriptionError, 0).expect("Error in sending a reply");
    }

    if init_config.decimals > 100 {
        msg::reply(FTError::DecimalsError, 0).expect("Error in sending a reply");
    }

    let mut balances = HashMap::new();
    balances.insert(init_config.admin, init_config.initial_supply);

    let ft = FungibleToken {
        name: init_config.name,
        symbol: init_config.symbol,
        decimals: init_config.decimals,
        description: init_config.description,
        external_links: init_config.external_links,
        current_supply: init_config.initial_supply,
        balances,
        admins: vec![init_config.admin],
        config: init_config.config,
        ..Default::default()
    };
    unsafe { FUNGIBLE_TOKEN = Some(ft) };

    msg::reply(FTEvent::Initialized, 0).expect("Error in sending a reply");
}

#[no_mangle]
extern "C" fn state() {
    let token = unsafe {
        FUNGIBLE_TOKEN
            .take()
            .expect("Unexpected: Error in getting contract state")
    };

    let query: FTQuery = msg::load().expect("Unable to decode the query");
    let reply = match query {
        FTQuery::Name => FTQueryReply::Name(token.name),
        FTQuery::Symbol => FTQueryReply::Symbol(token.symbol),
        FTQuery::Decimals => FTQueryReply::Decimals(token.decimals),
        FTQuery::Description => FTQueryReply::Description(token.description),
        FTQuery::ExternalLinks => FTQueryReply::ExternalLinks(token.external_links),
        FTQuery::CurrentSupply => FTQueryReply::CurrentSupply(token.current_supply),
        FTQuery::BalanceOf(account) => {
            let balance = if let Some(balance) = token.balances.get(&account) {
                *balance
            } else {
                0
            };
            FTQueryReply::Balance(balance)
        }
        FTQuery::AllowanceOfAccount {
            account,
            approved_account,
        } => {
            let allowance = if let Some(allowance) = token
                .allowances
                .get(&account)
                .and_then(|m| m.get(&approved_account))
            {
                *allowance
            } else {
                0
            };
            FTQueryReply::AllowanceOfAccount(allowance)
        }
        FTQuery::Admins => FTQueryReply::Admins(token.admins),
        FTQuery::GetTxValidityTime { account, tx_id } => {
            let valid_until = token.tx_ids.get(&(account, tx_id)).unwrap_or(&0);
            FTQueryReply::TxValidityTime(*valid_until)
        }
        FTQuery::GetTxIdsForAccount { account } => {
            let tx_ids: Vec<TxId> =
                if let Some(tx_ids) = token.account_to_tx_ids.get(&account).cloned() {
                    tx_ids.into_iter().collect()
                } else {
                    Vec::new()
                };
            FTQueryReply::TxIdsForAccount { tx_ids }
        }
    };
    msg::reply(reply, 0).expect("Error on sharinf state");
}