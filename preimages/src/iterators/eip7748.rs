//! Implementation of the EIP-7748 preimage access sequence iterator.
//!
//! This module provides an account and storage slot iterator respecting the order defined in EIP-7748.
//! The ordering can be summarized as:
//! 1. DFS the state tree, until an account is reached.
//! 2. For each account, iterate over its state trie also in DFS order.
//!
//! In summary, the ordering is based by account and storage slot _hash_ (i.e keccak256).
//!
//! Sample output: [hash(account1), hash(account1_ss0), hash(account1_ss1), hash(account2), hash(account3), hash(account3_ss0), ...]

use alloy_primitives::{keccak256, Address, B256};
use anyhow::Result;
use rayon::slice::ParallelSliceMut;
use reth_db::mdbx::cursor::Cursor;
use reth_db::mdbx::RO;
use reth_db::{mdbx::tx::Tx, PlainAccountState, PlainStorageState};
use reth_db_api::cursor::DbCursorRO;
use reth_db_api::transaction::DbTx;

use super::{AccountStorageItem, PreimageIterator};

pub struct Eip7748Iterator {
    state: State,

    ordered_addresses: Vec<Address>,
    ordered_addresses_idx: usize,

    cursor_storage_slots: Cursor<RO, PlainStorageState>,
    buf_storage_slot: Option<Vec<B256>>,
    buf_storage_slot_idx: usize,
}

enum State {
    Account,
    StorageSlot(Address),
    End,
}

impl PreimageIterator for Eip7748Iterator {}

impl Eip7748Iterator {
    pub fn new<P>(tx: &Tx<RO>, mut progress: Option<P>) -> Result<Self>
    where
        P: FnMut(Address),
    {
        let mut addresses = Vec::with_capacity(300_000_000);
        let mut cursor_accounts = tx.cursor_read::<PlainAccountState>()?;
        while let Some((address, _)) = cursor_accounts.next()? {
            addresses.push((address, keccak256(address)));
            if let Some(ref mut progress) = progress {
                progress(address);
            }
        }
        addresses.par_sort_by_key(|addr| addr.1);

        Ok(Eip7748Iterator {
            state: State::Account,
            ordered_addresses: addresses.into_iter().map(|(addr, _)| addr).collect(),
            ordered_addresses_idx: 0,
            cursor_storage_slots: tx.cursor_read::<PlainStorageState>()?,
            buf_storage_slot: None,
            buf_storage_slot_idx: 0,
        })
    }
}

impl Iterator for Eip7748Iterator {
    type Item = Result<AccountStorageItem>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.state {
            State::Account => match self.ordered_addresses.get(self.ordered_addresses_idx) {
                Some(address) => {
                    self.ordered_addresses_idx += 1;
                    self.state = State::StorageSlot(*address);
                    Some(Ok(AccountStorageItem::Account(*address)))
                }
                None => {
                    self.state = State::End;
                    None
                }
            },
            State::StorageSlot(address) => {
                let sorted_storage_slots = self.buf_storage_slot.get_or_insert_with(|| {
                    let mut storage_slots = Vec::with_capacity(1 << 15);
                    let mut curr = self.cursor_storage_slots.seek(*address).unwrap();
                    while let Some((addr, ss)) = curr {
                        if addr != *address {
                            break;
                        }
                        storage_slots.push((ss.key, keccak256(ss.key)));
                        curr = self.cursor_storage_slots.next().unwrap();
                    }
                    storage_slots.par_sort_by_key(|(_, hashed_ss)| *hashed_ss);
                    storage_slots.into_iter().map(|(ss, _)| ss).collect()
                });

                match sorted_storage_slots.get(self.buf_storage_slot_idx) {
                    Some(key) => {
                        self.buf_storage_slot_idx += 1;
                        Some(Ok(AccountStorageItem::StorageSlot(*address, *key)))
                    }
                    None => {
                        self.buf_storage_slot = None;
                        self.buf_storage_slot_idx = 0;
                        self.state = State::Account;
                        self.next()
                    }
                }
            }
            State::End => None,
        }
    }
}
