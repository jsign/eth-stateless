use anyhow::{anyhow, Context, Result};
use clap::{command, Args, Parser};
use iterators::{
    eip4762::Eip4762Iterator, plain::PlainIterator, AccountStorageItem, PreimageIterator,
};
use progress::AddressProgressBar;
use reth_db::mdbx::{tx::Tx, DatabaseArguments, MaxReadTransactionDuration};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

mod iterators;
mod progress;

#[derive(Parser)]
#[command(name = "report")]
struct Cli {
    #[arg(short = 'd', long = "datadir", help = "Reth datadir path")]
    datadir: String,

    #[command(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    #[command(name = "generate", about = "Generate preimage file")]
    Generate {
        #[arg(
            long = "output-path",
            help = "Preimages file output path",
            default_value = "preimages.bin"
        )]
        path: String,

        #[command(flatten)]
        order: OrderArgs,
    },
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct OrderArgs {
    #[arg(long, help = "Use plain ordering")]
    plain: bool,
    #[arg(long, help = "Use EIP-4762 ordering (i.e: hashed)")]
    eip4762: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let tx = Tx::new(
        reth_db::open_db_read_only(
            Path::new(&cli.datadir).join("db").as_path(),
            DatabaseArguments::default()
                .with_max_read_transaction_duration(Some(MaxReadTransactionDuration::Unbounded)),
        )
        .map_err(|e| anyhow!("Failed to open db: {}", e))?
        .begin_ro_txn()
        .context("opening tx")?,
    );

    match cli.subcmd {
        SubCommand::Generate {
            path,
            order: iterator,
        } => {
            if iterator.plain {
                println!("[1/1] Generating preimage file...");
                generate(
                    &path,
                    PlainIterator::new(tx)?,
                    AddressProgressBar::new(false),
                )?;
            } else {
                println!("[1/2] Ordering account addresses by hash...");
                let mut pb = AddressProgressBar::new(false);
                let it = Eip4762Iterator::new(tx, Some(|addr| pb.progress(addr)))?;
                println!("[2/2] Generating preimage file...");
                generate(&path, it, AddressProgressBar::new(true))?;
            }
        }
    }

    Ok(())
}

fn generate(path: &str, it: impl PreimageIterator, mut pb: AddressProgressBar) -> Result<()> {
    let mut f = BufWriter::new(File::create(path)?);
    for entry in it {
        match entry {
            Ok(AccountStorageItem::Account(address)) => {
                pb.progress(address);
                f.write_all(address.as_slice())
                    .context("writing address preimage")?;
            }
            Ok(AccountStorageItem::StorageSlot(ss)) => {
                f.write_all(ss.as_slice())
                    .context("writing storage slot preimage")?;
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
