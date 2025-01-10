//! Implementation of the plain preimage access sequence iterator.
//!
//! This module provides an account and storage slot iterator respecting the plain ordering in the database.
//! The ordering can be summarized as:
//! 1. Iterate the account sorted by address.
//! 2. For each account, iterate over the sorted storage slots.
//!
//! No actual sorting is required since both addresses and storage slots are naturally sorted in the db.
//!
//! Sample output: [account1, account1_ss0, account1_ss1, account2, account3, account3_ss0, ...]

use alloy_primitives::{Address, B256};
use anyhow::Result;
use reth_db::mdbx::cursor::Cursor;
use reth_db::mdbx::RO;
use reth_db::{mdbx::tx::Tx, PlainAccountState, PlainStorageState};
use reth_db_api::cursor::DbCursorRO;
use reth_db_api::transaction::DbTx;

use super::{AccountStorageItem, PreimageIterator};

pub struct PlainIterator {
    cursor_accounts: Cursor<RO, PlainAccountState>,
    cursor_storage_slots: Cursor<RO, PlainStorageState>,

    state: State,
    buf_storage_slot: Option<(Address, B256)>,
}

enum State {
    Account,
    StorageSlot(Address),
    End,
}

impl PlainIterator {
    pub fn new(tx: &Tx<RO>) -> Result<Self> {
        let cursor_accounts = tx.cursor_read::<PlainAccountState>()?;
        let cursor_storage_slots = tx.cursor_read::<PlainStorageState>()?;

        Ok(PlainIterator {
            cursor_accounts,
            cursor_storage_slots,
            state: State::Account,
            buf_storage_slot: None,
        })
    }
}

impl PreimageIterator for PlainIterator {}

impl Iterator for PlainIterator {
    type Item = Result<AccountStorageItem>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.state {
            State::Account => {
                let next_account = match self.cursor_accounts.next() {
                    Ok(account) => account,
                    Err(e) => return Some(Err(e.into())),
                };
                match next_account {
                    Some((address, _)) => {
                        self.state = State::StorageSlot(address);
                        Some(Ok(AccountStorageItem::Account(address)))
                    }
                    None => {
                        self.state = State::End;
                        None
                    }
                }
            }
            State::StorageSlot(address) => {
                if let Some((addr, key)) = self.buf_storage_slot {
                    if addr == *address {
                        self.buf_storage_slot = None;
                        return Some(Ok(AccountStorageItem::StorageSlot(addr, key)));
                    } else {
                        self.state = State::Account;
                        return self.next();
                    }
                }
                let next_storage_slot = match self.cursor_storage_slots.next() {
                    Ok(storage_entry) => storage_entry,
                    Err(e) => return Some(Err(e.into())),
                };
                match next_storage_slot {
                    Some((addr, storage_entry)) => {
                        self.buf_storage_slot = Some((addr, storage_entry.key));
                        self.next()
                    }
                    None => {
                        self.state = State::Account;
                        self.next()
                    }
                }
            }
            State::End => None,
        }
    }
}
