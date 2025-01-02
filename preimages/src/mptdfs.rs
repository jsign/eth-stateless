use alloy_primitives::{Address, B256};
use anyhow::Result;
use reth_db::mdbx::cursor::Cursor;
use reth_db::mdbx::RO;
use reth_db::{mdbx::tx::Tx, PlainAccountState, PlainStorageState};
use reth_db_api::cursor::DbCursorRO;
use reth_db_api::transaction::DbTx;

pub struct MptDfsIterator {
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

impl MptDfsIterator {
    pub fn new(tx: Tx<RO>) -> Result<Self> {
        let cursor_accounts = tx.cursor_read::<PlainAccountState>()?;
        let cursor_storage_slots = tx.cursor_read::<PlainStorageState>()?;

        Ok(MptDfsIterator {
            cursor_accounts,
            cursor_storage_slots,
            state: State::Account,
            buf_storage_slot: None,
        })
    }
}

pub enum MptDfsItem {
    Account(Address),
    StorageSlot(B256),
}

impl Iterator for MptDfsIterator {
    type Item = MptDfsItem;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.state {
            State::Account => {
                let next_account = self.cursor_accounts.next().unwrap();
                match next_account {
                    Some((address, _)) => {
                        self.state = State::StorageSlot(address);
                        Some(MptDfsItem::Account(address))
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
                        return Some(MptDfsItem::StorageSlot(key));
                    } else {
                        self.state = State::Account;
                        return self.next();
                    }
                }
                let next_storage_slot = self.cursor_storage_slots.next().unwrap();
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
