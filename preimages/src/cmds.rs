use crate::iterators::plain::PlainIterator;
use crate::iterators::{AccountStorageItem, PreimageIterator};
use crate::progress::AddressProgressBar;
use alloy_primitives::{Address, FixedBytes};
use anyhow::{anyhow, Context, Result};
use reth_db::mdbx::tx::Tx;
use reth_db::mdbx::RO;
use std::collections::HashMap;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
};

pub fn generate(path: &str, it: impl PreimageIterator, mut pb: AddressProgressBar) -> Result<()> {
    let mut f = BufWriter::new(File::create(path)?);
    for entry in it {
        match entry {
            Ok(AccountStorageItem::Account(address)) => {
                pb.progress(address);
                f.write_all(address.as_slice())
                    .context("writing address preimage")?;
            }
            Ok(AccountStorageItem::StorageSlot(_, ss)) => {
                f.write_all(ss.as_slice())
                    .context("writing storage slot preimage")?;
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

pub fn verify(path: &str, it: impl PreimageIterator, mut pb: AddressProgressBar) -> Result<()> {
    let mut reader = BufReader::new(File::open(path)?);

    for entry in it {
        match entry {
            Ok(AccountStorageItem::Account(addr)) => {
                pb.progress(addr);
                let mut file_addr: Address = Default::default();
                reader
                    .read_exact(file_addr.as_mut_slice())
                    .context("reading address preimage")?;

                if addr != file_addr.as_slice() {
                    return Err(anyhow!("Address {} preimage mismatch", file_addr));
                }
            }
            Ok(AccountStorageItem::StorageSlot(address, ss)) => {
                let mut file_ss: FixedBytes<32> = Default::default();
                reader
                    .read_exact(file_ss.as_mut_slice())
                    .context("reading storage slot preimage")?;
                if ss != file_ss.as_slice() {
                    return Err(anyhow!(
                        "Storage slot {} preimage (address: {}) mistmatch",
                        ss,
                        address
                    ));
                }
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

pub fn storage_slot_freq<const N: usize>(tx: &Tx<RO>, top_n_detail: usize) -> Result<()> {
    let mut counts: HashMap<[u8; N], u32> = HashMap::new();
    let mut pb = AddressProgressBar::new(false);
    let it = PlainIterator::new(tx)?;
    let mut total_storage_slots = 0;
    for entry in it {
        match entry {
            Ok(AccountStorageItem::Account(address)) => {
                pb.progress(address);
            }
            Ok(AccountStorageItem::StorageSlot(_, key)) => {
                total_storage_slots += 1;
                counts
                    .entry(key.0[0..N].try_into()?)
                    .and_modify(|e| *e += 1)
                    .or_insert(1);
                if counts.len() > 200_000_000 {
                    counts.retain(|_, count| *count > 1);
                }
            }
            Err(e) => return Err(e),
        }
    }
    // Only keep storage slots that are _potentially_ worth deduping.
    counts.retain(|_, count| *count > 1);

    let mut counts_vec = counts.iter().collect::<Vec<_>>();
    counts_vec.sort_unstable_by_key(|(_, v)| std::cmp::Reverse(*v));
    let mut cummulative_count = 0;
    println!(
        "Top {} storage slot {}-byte prefix repetitions:",
        top_n_detail, N
    );
    for e in counts_vec.iter().take(top_n_detail) {
        cummulative_count += e.1;
        println!(
            "{}: {} ({:.2}%) ~{}MiB (cumm {:.2}MiB)",
            hex::encode(e.0),
            e.1,
            (*e.1 as f64) / (total_storage_slots as f64) * 100.0,
            e.1 * (N as u32) / 1024 / 1024,
            cummulative_count * (N as u32) / 1024 / 1024,
        );
    }

    Ok(())
}
