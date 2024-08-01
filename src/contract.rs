use gstd::{
    collections::{
        HashMap, 
        HashSet
    }, 
    msg, 
    ActorId, 
    String, 
    Vec
};
use io::{
    Config, 
    ExternalLinks, 
    FTError, 
    FTEvent, 
    TxId, 
    ValidUntil
};

use crate::ZERO_ID;

#[derive(Default)]
pub struct FungibleToken {
    /// Name of the token.
    pub name: String,
    /// Symbol of the token.
    pub symbol: String,
    /// Token's decimals.
    pub decimals: u8,
    /// Description of the token
    pub description: String,
    /// ExternalLinks
    pub external_links: ExternalLinks,
    /// Current supply of the token.
    pub current_supply: u128,
    /// Map to hold balances of token holders.
    pub balances: HashMap<ActorId, u128>,
    /// Map to hold allowance information of token holders.
    pub allowances: HashMap<ActorId, HashMap<ActorId, u128>>,
    /// Mapping of executed transactions to the time they are valid.
    pub tx_ids: HashMap<(ActorId, TxId), ValidUntil>,
    /// Mapping of accounts to their transaction IDs.
    pub account_to_tx_ids: HashMap<ActorId, HashSet<TxId>>,
    /// Configuration parameters for the fungible token contract.
    pub config: Config,
    pub admins: Vec<ActorId>,
    pub liquidity_contract: ActorId
}

impl FungibleToken {
    pub fn add_contract(&mut self, contract: &ActorId) -> Result<FTEvent, FTError> { 
        let source: ActorId = msg::source();

        if !self.liquidity_contract.is_zero() {
            Err(FTError::CantDeleteYourself)
        }

        if !self.admins.contains(&source) {
            Err(FTError::NotAdmin)
        }
        
        self.liquidity_contract = *contract;
        Ok(FTEvent::AdminAdded { 
            admin_id: *contract 
        })
    }

    pub fn transfer_to_users(&mut self, amount: u128, to_users: Vec<ActorId>) -> Result<FTEvent, FTError> {
        let source = msg::source();

        if !self.admins.contains(&source) {
            return Err(FTError::NotAdmin)
        }

        self.check_balance(&source, amount * to_users.len() as u128)?;

        for to in to_users.clone() {
            self.balances
                .entry(source)
                .and_modify(|balance| *balance -= amount);
            self.balances
                .entry(to)
                .and_modify(|balance| *balance += amount)
                .or_insert(amount);
        }

        Ok(FTEvent::TransferredToUsers {
            from: source,
            to_users,
            amount,
        })
    }

    pub fn mint(&mut self, amount: u128, to: ActorId) -> Result<FTEvent, FTError> {
        if self.admins.contains(&msg::source()) {
            self.balances
                .entry(to)
                .and_modify(|balance| *balance += amount)
                .or_insert(amount);
            
            self.current_supply += amount;

            return Ok(FTEvent::Minted {
                to,
                amount
            });
        }

        return Err(FTError::NotAdmin);
    } 

    pub fn burn(&mut self, amount: u128) -> Result<FTEvent, FTError> {
        let source = msg::source();
        if self.balances.get(&source).unwrap_or(&0) < &amount {
            return Err(FTError::NotEnoughBalance);
        }
        self.balances
            .entry(source)
            .and_modify(|balance| *balance -= amount);

        self.current_supply -= amount;

        Ok(FTEvent::Transferred {
            from: source,
            to: ZERO_ID,
            amount,
        })
    }

    pub fn add_admin(&mut self, admin_id: &ActorId) -> Result<FTEvent, FTError> {
        let source = msg::source();

        if !self.admins.contains(&source) {
            return Err(FTError::NotAdmin);
        }

        if self.admins.contains(admin_id) {
            return Err(FTError::AdminAlreadyExists);
        }
        
        self.admins.push(*admin_id);
        Ok(FTEvent::AdminAdded {
            admin_id: *admin_id,
        })
    }

    pub fn delete_admin(&mut self, admin_id: &ActorId) -> Result<FTEvent, FTError> {
        let source = msg::source();
        if !self.admins.contains(&source) {
            return Err(FTError::NotAdmin);
        }

        if admin_id == &source {
            return Err(FTError::CantDeleteYourself);
        }

        self.admins.retain(|acc| acc != admin_id);
        Ok(FTEvent::AdminRemoved {
            admin_id: *admin_id,
        })
    }

    pub fn transfer(
        &mut self,
        from: &ActorId,
        to: &ActorId,
        amount: u128,
    ) -> Result<FTEvent, FTError> {
        let msg_source = msg::source();

        if *from == ActorId::zero() || *to == ActorId::zero() {
            return Err(FTError::ZeroAddress);
        };

        self.check_balance(from, amount)?;

        if &msg_source != self.liquidity_contract {
            self.can_transfer(&msg_source, from, amount)?;
        }

        self.balances
            .entry(*from)
            .and_modify(|balance| *balance -= amount);
            
        self.balances
            .entry(*to)
            .and_modify(|balance| *balance += amount)
            .or_insert(amount);

        Ok(FTEvent::Transferred {
            from: *from,
            to: *to,
            amount,
        })
    }

    /// Executed on receiving `fungible-token-messages::ApproveInput`.
    pub fn approve(
        &mut self,
        to: &ActorId,
        amount: u128,
    ) -> Result<FTEvent, FTError> {
        if *to == ActorId::zero() {
            return Err(FTError::ZeroAddress);
        }
        let msg_source = msg::source();

        self.allowances
            .entry(msg_source)
            .or_default()
            .insert(*to, amount);

        Ok(FTEvent::Approved {
            from: msg_source,
            to: *to,
            amount,
        })
    }

    fn check_balance(
        &self, 
        account: &ActorId, 
        amount: u128
    ) -> Result<(), FTError> {
        if *self.balances.get(account).unwrap_or(&0) < amount {
            return Err(FTError::NotEnoughBalance)
        }
        Ok(())
    }

    fn can_transfer(
        &mut self, 
        source: &ActorId, 
        from: &ActorId, 
        amount: u128
    ) -> Result<(), FTError> {
        if source != from {
            if let Some(allowed_amount) = self.allowances.get(from).and_then(|m| m.get(source)) {
                if allowed_amount >= &amount {
                    self.allowances.entry(*from).and_modify(|m| {
                        m.entry(*source).and_modify(|a| *a -= amount);
                    });
                } else {
                    return Err(FTError::NotAllowedToTransfer);
                }
            } else {
                return Err(FTError::NotAllowedToTransfer);
            }
        }
        Ok(())
    }
}