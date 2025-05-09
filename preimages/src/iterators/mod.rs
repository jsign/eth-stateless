//! Multiple iterator implementators to dump the preimages in different orders
//!
//! This crate provides two different implementations of the preimage iterator:
//! - EIP-7748: The iterator respects the order defined in EIP-7748.
//! - Plain: The iterator respects the plain ordering in the database.
//!
//! See each module docs for more information.

use alloy_primitives::{Address, B256};
use anyhow::Result;

pub mod eip7748;
pub mod plain;

pub enum AccountStorageItem {
    Account(Address),
    StorageSlot(Address, B256),
}
pub trait PreimageIterator: Iterator<Item = Result<AccountStorageItem>> {}
