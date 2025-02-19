use std::sync::LazyLock;

use alloy_primitives::{b256, Address};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use reth_db::mdbx::tx::Tx;
use reth_db::mdbx::RO;
use reth_db::{Bytecodes, PlainAccountState, PlainStorageState};
use reth_db_api::cursor::DbCursorRO;
use reth_db_api::transaction::DbTx;
use tabled::Tabled;

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

pub struct AccountStorage {
    pub address: Address,
    pub total_slots: u64,
}

pub fn storage_slots_stats(tx: &Tx<RO>) -> Result<Vec<AccountStorage>> {
    let bar = ProgressBar::new(tx.entries::<PlainStorageState>()? as u64)
        .with_style(PROGRESS_STYLE.clone())
        .with_message("Analyzing storage slots...");

    let mut contracts = Vec::<AccountStorage>::new();
    let mut cur = tx.cursor_read::<PlainStorageState>()?;
    loop {
        match cur.next() {
            Ok(Some((address, _))) => {
                if contracts.is_empty() || address != contracts.last().unwrap().address {
                    contracts.push(AccountStorage {
                        address,
                        total_slots: 0,
                    });
                }
                let contract = contracts.last_mut().unwrap();
                contract.total_slots += 1;

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
