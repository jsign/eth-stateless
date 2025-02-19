use std::sync::LazyLock;

use alloy_primitives::{b256, Address, B256, U256};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use reth_db::mdbx::tx::Tx;
use reth_db::mdbx::RO;
use reth_db::{Bytecodes, PlainAccountState, PlainStorageState};
use reth_db_api::cursor::DbCursorRO;
use reth_db_api::transaction::DbTx;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use tabled::Tabled;

use crate::main;

static PROGRESS_STYLE: LazyLock<ProgressStyle> = LazyLock::new(|| {
    ProgressStyle::with_template("{bar:50.cyan/blue} {percent}% [eta: {eta}] {msg}")
        .expect("Failed to set progress bar style template")
        .progress_chars("#>-")
});

#[derive(Debug, Tabled)]
pub struct Stats {
    pub average: u64,
    pub median: u64,
    pub p99: u64,
    pub max: u64,
}

pub fn account_stats(tx: &Tx<RO>) -> Result<(u64, u64, Stats)> {
    let bar = ProgressBar::new(tx.entries::<PlainAccountState>()? as u64)
        .with_style(PROGRESS_STYLE.clone())
        .with_message("Analyzing accounts...");

    let mut code_lens = Vec::<u64>::new();
    let mut eoa_count = 0u64;
    let mut contract_count = 0u64;

    let mut cur = tx.cursor_read::<PlainAccountState>()?;
    loop {
        match cur.next() {
            Ok(Some((_, account))) => {
                let bytecode_hash = account.get_bytecode_hash();
                if bytecode_hash
                    == b256!("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470")
                {
                    eoa_count += 1;
                } else {
                    contract_count += 1;
                    code_lens.push(
                        tx.get::<Bytecodes>(bytecode_hash)?
                            .unwrap()
                            .len()
                            .try_into()?,
                    );
                }
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

    Ok((eoa_count, contract_count, calculate_stats(&mut code_lens)))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountStemStats {
    pub address: Address,
    pub account_stem: u16,
    pub ss_count: u64,
    pub ss_stems: Vec<u16>,
    pub code_stems: u16,
}

pub fn stem_stats(tx: &Tx<RO>, group_size: u16) -> Result<Vec<AccountStemStats>> {
    let header_storage_offset = 64;
    let code_offset = group_size / 2;
    let ss_header_count = to_b256(code_offset - header_storage_offset);
    let group_size_bits = group_size.trailing_zeros();

    let bar = ProgressBar::new(tx.entries::<PlainStorageState>()? as u64)
        .with_style(PROGRESS_STYLE.clone())
        .with_message("Analyzing storage slots...");

    let mut contracts = Vec::<AccountStemStats>::new();
    let mut cur = tx.cursor_read::<PlainStorageState>()?;
    let mut curr_ss_group = U256::default();
    loop {
        match cur.next() {
            Ok(Some((address, slot))) => {
                if contracts.is_empty() || address != contracts.last().unwrap().address {
                    curr_ss_group = U256::default();

                    let account = tx.get::<PlainAccountState>(address)?.unwrap();
                    let bytecode = tx
                        .get::<Bytecodes>(account.get_bytecode_hash())?
                        .unwrap_or_default();
                    let code_chunks_count = ((bytecode.len() + 30) / 31) as u16;
                    let code_chunks_in_header = min(group_size - code_offset, code_chunks_count);

                    contracts.push(AccountStemStats {
                        address,
                        account_stem: 1 + 1 + code_chunks_in_header, // BASIC_DATA + CODE_HASH + header_code_chunks
                        ss_count: 0,
                        ss_stems: vec![],
                        code_stems: code_chunks_count - code_chunks_in_header,
                    });
                }
                let contract = contracts.last_mut().unwrap();
                contract.ss_count += 1;

                if slot.key < ss_header_count {
                    contract.account_stem += 1;
                } else {
                    let (mut ss_group, _) = U256::from_be_slice(slot.key.as_slice())
                        .overflowing_shr(group_size_bits as usize);
                    ss_group = ss_group.checked_add(U256::from(1)).unwrap();

                    if ss_group != curr_ss_group {
                        curr_ss_group = ss_group;
                        contract.ss_stems.push(1);
                    } else {
                        *contract.ss_stems.last_mut().unwrap() += 1;
                    }
                    contract.ss_stems.push(1);
                }

                bar.inc(1);
            }
            Ok(None) => {
                break;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }
    bar.finish_and_clear();

    Ok(contracts)
}

fn calculate_stats(data: &mut [u64]) -> Stats {
    data.sort();
    let sum: u64 = data.iter().sum();
    let average = sum / data.len() as u64;
    let median = data[data.len() / 2];
    let p99 = data[(data.len() as f64 * 0.99) as usize];
    let max = *data.last().unwrap();

    Stats {
        average,
        median,
        p99,
        max,
    }
}

fn to_b256(value: u16) -> B256 {
    let mut buf = [0u8; 32];
    buf[30..].copy_from_slice(&value.to_be_bytes());
    B256::new(buf)
}
