use crate::iterators::{AccountStorageItem, PreimageIterator};
use crate::progress::AddressProgressBar;
use alloy_primitives::{Address, FixedBytes};
use anyhow::{anyhow, Context, Result};
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
