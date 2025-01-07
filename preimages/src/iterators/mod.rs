use alloy_primitives::{Address, B256};
use anyhow::Result;

pub mod eip4762;
pub mod plain;

pub enum AccountStorageItem {
    Account(Address),
    StorageSlot(B256),
}
pub trait PreimageIterator: Iterator<Item = Result<AccountStorageItem>> {}
