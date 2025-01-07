use anyhow::{Context, Result};
use clap::{command, Args, Parser};
use iterators::{
    eip4762::Eip4762Iterator, plain::PlainIterator, AccountStorageItem, PreimageIterator,
};
use progress::PreimagesProgressBar;
use reth_db::mdbx::{tx::Tx, DatabaseArguments};
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
    let datadir = Path::new(&cli.datadir);

    let db = reth_db::open_db_read_only(
        datadir.join("db").as_path(),
        DatabaseArguments::default().with_max_read_transaction_duration(Some(
            reth_db::mdbx::MaxReadTransactionDuration::Unbounded,
        )),
    )
    .unwrap();
    let tx = db.begin_ro_txn().context("opening tx")?;
    let tx = Tx::new(tx);

    match cli.subcmd {
        SubCommand::Generate {
            path,
            order: iterator,
        } => {
            if iterator.plain {
                generate(&path, PlainIterator::new(tx)?)?;
            } else {
                generate(&path, Eip4762Iterator::new(tx)?)?;
            }
        }
    }

    Ok(())
}

fn generate(path: &str, it: impl PreimageIterator) -> Result<()> {
    let mut f = File::create(path)?;
    let mut writer = BufWriter::new(&mut f);

    let mut pb = PreimagesProgressBar::new()?;
    for entry in it {
        match entry {
            Ok(AccountStorageItem::Account(address)) => {
                pb.progress(address);
                writer
                    .write_all(address.as_slice())
                    .context("writing address preimage")?;
            }
            Ok(AccountStorageItem::StorageSlot(ss)) => {
                writer
                    .write_all(ss.as_slice())
                    .context("writing storage slot preimage")?;
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
