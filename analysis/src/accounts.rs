use std::sync::LazyLock;

use alloy_primitives::{Address, B256, U256};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use reth_db::mdbx::tx::Tx;
use reth_db::mdbx::RO;
use reth_db::{Bytecodes, PlainAccountState, PlainStorageState};
use reth_db_api::cursor::DbCursorRO;
use reth_db_api::transaction::DbTx;
use serde::{Deserialize, Serialize};
use std::cmp::min;

static PROGRESS_STYLE: LazyLock<ProgressStyle> = LazyLock::new(|| {
    ProgressStyle::with_template("{bar:50.cyan/blue} {percent}% [eta: {eta}] {msg}")
        .expect("Failed to set progress bar style template")
        .progress_chars("#>-")
});

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountStemStats {
    pub address: Address,
    pub bytecode_len: usize,
    pub account_stem: u16,
    pub ss_stems: Vec<u16>,
    pub code_stems: u16,
    pub num_storage_slots: usize,
}

pub fn account_stats(tx: &Tx<RO>, group_size: u16) -> Result<Vec<AccountStemStats>> {
    let bar = ProgressBar::new(tx.entries::<PlainAccountState>()? as u64)
        .with_style(PROGRESS_STYLE.clone())
        .with_message("Analyzing...");

    let header_storage_offset = 64;
    let code_offset = group_size / 2;
    let ss_header_count = to_b256(code_offset - header_storage_offset);
    let group_size_bits = group_size.trailing_zeros();

    let mut accounts = Vec::<AccountStemStats>::new();
    let mut cur = tx.cursor_read::<PlainAccountState>()?;
    loop {
        match cur.next() {
            Ok(Some((address, _))) => {
                bar.set_message(address.to_string().to_lowercase());
                let account = tx.get::<PlainAccountState>(address)?.unwrap();
                let bytecode = tx
                    .get::<Bytecodes>(account.get_bytecode_hash())?
                    .unwrap_or_default();
                let code_chunks_count = ((bytecode.len() + 30) / 31) as u16;
                let code_chunks_in_header = min(group_size - code_offset, code_chunks_count);

                let mut stats = AccountStemStats {
                    address,
                    bytecode_len: bytecode.len(),
                    account_stem: 1 + 1 + code_chunks_in_header, // BASIC_DATA + CODE_HASH + header_code_chunks
                    ss_stems: vec![],
                    code_stems: (code_chunks_count - code_chunks_in_header).div_ceil(group_size),
                    num_storage_slots: 0,
                };

                let mut cur = tx.cursor_read::<PlainStorageState>()?;
                let mut entry = cur.seek_exact(address)?;
                let mut curr_ss_group = U256::default();
                while let Some((slot_address, slot)) = entry {
                    if slot_address != address {
                        break;
                    }
                    stats.num_storage_slots += 1;
                    if slot.key < ss_header_count {
                        stats.account_stem += 1;
                    } else {
                        let (mut ss_group, _) = U256::from_be_slice(slot.key.as_slice())
                            .overflowing_shr(group_size_bits as usize);
                        ss_group = ss_group.checked_add(U256::from(1)).unwrap();

                        if ss_group != curr_ss_group {
                            curr_ss_group = ss_group;
                            stats.ss_stems.push(1);
                        } else {
                            *stats.ss_stems.last_mut().unwrap() += 1;
                        }
                    }
                    entry = cur.next()?;
                }
                accounts.push(stats);
            }
            Ok(None) => {
                break;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
        bar.inc(1);
    }
    bar.finish_and_clear();

    Ok(accounts)
}

fn to_b256(value: u16) -> B256 {
    let mut buf = [0u8; 32];
    buf[30..].copy_from_slice(&value.to_be_bytes());
    B256::new(buf)
}
