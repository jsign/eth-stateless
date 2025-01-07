use alloy_primitives::{Address, B256};
use anyhow::Result;
use reth_db::mdbx::cursor::Cursor;
use reth_db::mdbx::RO;
use reth_db::{mdbx::tx::Tx, PlainAccountState, PlainStorageState};
use reth_db_api::cursor::DbCursorRO;
use reth_db_api::transaction::DbTx;

pub struct UnhashedIterator {
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

impl UnhashedIterator {
    pub fn new(tx: Tx<RO>) -> Result<Self> {
        let cursor_accounts = tx.cursor_read::<PlainAccountState>()?;
        let cursor_storage_slots = tx.cursor_read::<PlainStorageState>()?;

        Ok(UnhashedIterator {
            cursor_accounts,
            cursor_storage_slots,
            state: State::Account,
            buf_storage_slot: None,
        })
    }
}

pub enum AccountStorageItem {
    Account(Address),
    StorageSlot(B256),
}

impl Iterator for UnhashedIterator {
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
                        self.next()
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
                        return Some(Ok(AccountStorageItem::StorageSlot(key)));
                    } else {
                        let curr_addr = *address;
                        self.state = State::Account;
                        return Some(Ok(AccountStorageItem::Account(curr_addr)));
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
                        let curr_addr = *address;
                        self.state = State::Account;
                        Some(Ok(AccountStorageItem::Account(curr_addr)))
                    }
                }
            }
            State::End => None,
        }
    }
}
