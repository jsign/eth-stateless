use alloy_primitives::b256;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use reth_db::mdbx::{tx::Tx, TransactionKind};
use reth_db::{Bytecodes, PlainAccountState, PlainStorageState};
use reth_db_api::cursor::DbCursorRO;
use reth_db_api::transaction::DbTx;
use serde::{Deserialize, Serialize};

const PROGRESS_STYLE: &str = "{msg} [{bar:40.cyan/blue}] {pos}/{len} [{elapsed_precise}] ({eta})";
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Report {
    eoa_count: u64,
    contract_count: u64,

    code_stats: Stats,
    storage_slots_stats: Stats,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Stats {
    average: u64,
    median: u64,
    p99: u64,
}

pub fn generate(tx: &mut Tx<impl TransactionKind>) -> Result<Report> {
    let (eoa_count, contract_count, code_stats) = account_stats(tx)?;
    let storage_slots_stats = storage_slots_stats(tx)?;

    Ok(Report {
        eoa_count,
        contract_count,
        code_stats,
        storage_slots_stats,
    })
}

fn account_stats(tx: &mut Tx<impl TransactionKind>) -> Result<(u64, u64, Stats)> {
    let bar = ProgressBar::new(tx.entries::<PlainAccountState>()? as u64)
        .with_style(ProgressStyle::with_template(PROGRESS_STYLE).unwrap())
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
    bar.finish();

    Ok((eoa_count, contract_count, calculate_stats(&mut code_lens)))
}

fn storage_slots_stats(tx: &mut Tx<impl TransactionKind>) -> Result<Stats> {
    let bar = ProgressBar::new(tx.entries::<PlainStorageState>()? as u64)
        .with_style(ProgressStyle::with_template(PROGRESS_STYLE).unwrap())
        .with_message("Analyzing storage slots...");

    let mut addresses_ss_count = Vec::<u64>::new();
    let mut current_addr = None;
    let mut curr_count = 0;
    let mut cur = tx.cursor_read::<PlainStorageState>()?;
    loop {
        match cur.next() {
            Ok(Some((address, _))) => {
                bar.inc(1);
                match current_addr {
                    Some(curr_addr) if curr_addr != address => {
                        addresses_ss_count.push(curr_count);
                        current_addr = Some(address);
                        curr_count = 1;
                    }
                    Some(_) => {
                        curr_count += 1;
                    }
                    None => {
                        current_addr = Some(address);
                        curr_count = 1;
                    }
                }
            }
            Ok(None) => {
                break;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }
    if current_addr.is_some() {
        addresses_ss_count.push(curr_count);
    }
    bar.finish();

    Ok(calculate_stats(&mut addresses_ss_count))
}

fn calculate_stats(data: &mut [u64]) -> Stats {
    data.sort();
    let sum: u64 = data.iter().sum();
    let average = sum / data.len() as u64;
    let median = data[data.len() / 2];
    let p99 = data[(data.len() as f64 * 0.99) as usize];

    Stats {
        average,
        median,
        p99,
    }
}
