use crate::iterators::plain::PlainIterator;
use crate::iterators::{AccountStorageItem, PreimageIterator};
use crate::progress::AddressProgressBar;
use alloy_primitives::{Address, FixedBytes};
use anyhow::{anyhow, Context, Result};
use reth_db::mdbx::tx::Tx;
use reth_db::mdbx::RO;
use std::collections::{BinaryHeap, HashMap};
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

pub fn storage_slot_freq(tx: Tx<RO>, n: usize) -> Result<()> {
    let mut counts: HashMap<FixedBytes<32>, i32> = HashMap::new();
    let mut pb = AddressProgressBar::new(false);
    let it = PlainIterator::new(tx)?;
    for entry in it {
        match entry {
            Ok(AccountStorageItem::Account(address)) => {
                pb.progress(address);
            }
            Ok(AccountStorageItem::StorageSlot(_, key)) => {
                counts.entry(key).and_modify(|e| *e += 1).or_insert(1);
                if counts.len() > 150_000_000 {
                    counts.retain(|_, count| *count > 1);
                }
            }
            Err(e) => return Err(e),
        }
    }

    let mut heap = BinaryHeap::new();
    for (&num, &count) in counts.iter() {
        heap.push((-count, num));
        if heap.len() > n {
            heap.pop();
        }
    }

    let mut top_n = Vec::with_capacity(n);
    while let Some((_, key)) = heap.pop() {
        top_n.push(key);
    }
    let mut sum = 0;

    println!("Top {} most repeated storage slots:", n);
    for key in top_n.iter().rev() {
        println!("{}: {}", key, counts[key]);
        sum += counts[key];
    }
    println!("Total count: {}", sum);
    println!("Total size: {}*32~={} MiB", n, sum * 32 / 1024 / 1024);

    Ok(())
}
