#![no_std]
use gmeta::{InOut, Metadata};
use gstd::{ActorId, Decode, Encode, String, TypeInfo, Vec};

pub type TxId = u64;
pub type ValidUntil = u64;

pub struct FungibleTokenMetadata;

impl Metadata for FungibleTokenMetadata {
    type Init = InOut<InitFt, Result<FTEvent, FTError>>;
    type Handle = InOut<FTAction, Result<FTEvent, FTError>>;
    type Reply = ();
    type Others = ();
    type Signal = ();
    type State = InOut<FTQuery, FTQueryReply>;
}

#[derive(TypeInfo, Encode, Decode)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct InitFt {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub description: String,
    pub external_links: ExternalLinks,
    pub initial_supply: u128,
    pub admin: ActorId,
    pub config: Config,
}

#[derive(TypeInfo, Encode, Decode, Default)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct ExternalLinks {
    twitter: String   
}

#[derive(TypeInfo, Encode, Decode, Default)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct Config {
    pub tx_storage_period: u64,
    pub tx_payment: u128
}

#[derive(TypeInfo, Encode, Decode)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum FTAction {
    TransferToUsers {
        amount: u128,
        to_users: Vec<ActorId>,
    },
    Mint {
        amount: u128,
        to: ActorId,
    },
    Burn {
        amount: u128,
    },
    Transfer {
        tx_id: Option<TxId>,
        from: ActorId,
        to: ActorId,
        amount: u128,
    },
    Approve {
        tx_id: Option<TxId>,
        to: ActorId,
        amount: u128,
    },
    BalanceOf(ActorId),
    AddAdmin {
        admin_id: ActorId,
    },
    DeleteAdmin {
        admin_id: ActorId,
    },
}

#[derive(TypeInfo, Encode, Decode)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum FTEvent {
    Initialized,
    TransferredToUsers {
        from: ActorId,
        to_users: Vec<ActorId>,
        amount: u128,
    },
    Transferred {
        from: ActorId,
        to: ActorId,
        amount: u128,
    },
    Approved {
        from: ActorId,
        to: ActorId,
        amount: u128,
    },
    AdminAdded {
        admin_id: ActorId,
    },
    AdminRemoved {
        admin_id: ActorId,
    },
    Balance(u128),
}

#[derive(TypeInfo, Encode, Decode)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum FTError {
    DecimalsError,
    DescriptionError,
    MaxSupplyReached,
    SupplyError,
    NotAdmin,
    NotEnoughBalance,
    ZeroAddress,
    NotAllowedToTransfer,
    AdminAlreadyExists,
    CantDeleteYourself,
    TxAlreadyExists,
}

#[derive(TypeInfo, Encode, Decode)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum FTQuery {
    Name,
    Symbol,
    Decimals,
    CurrentSupply,
    Description,
    ExternalLinks,
    BalanceOf(ActorId),
    AllowanceOfAccount {
        account: ActorId,
        approved_account: ActorId,
    },
    Admins,
    GetTxValidityTime {
        account: ActorId,
        tx_id: TxId,
    },
    GetTxIdsForAccount {
        account: ActorId,
    },
}

#[derive(TypeInfo, Encode, Decode)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum FTQueryReply {
    Name(String),
    Symbol(String),
    Decimals(u8),
    Description(String),
    ExternalLinks(ExternalLinks),
    CurrentSupply(u128),
    TotalSupply(u128),
    Balance(u128),
    AllowanceOfAccount(u128),
    Admins(Vec<ActorId>),
    TxValidityTime(ValidUntil),
    TxIdsForAccount { tx_ids: Vec<TxId> },
}

