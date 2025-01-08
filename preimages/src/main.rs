use anyhow::{anyhow, Context, Result};
use clap::{command, Args, Parser};
use iterators::{eip4762::Eip4762Iterator, plain::PlainIterator};
use progress::AddressProgressBar;
use reth_db::mdbx::{tx::Tx, DatabaseArguments, MaxReadTransactionDuration, RO};
use std::path::Path;

mod cmds;
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

    #[command(name = "verify", about = "Verify preimage file")]
    Verify {
        #[arg(
            short = 'i',
            long = "preimages-file-path",
            help = "Preimages file path",
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
        SubCommand::Generate { path, order } => generate_cmd(tx, &path, order)?,
        SubCommand::Verify { path, order } => {
            verify_cmd(tx, &path, order)?;
        }
    }

    Ok(())
}

fn generate_cmd(tx: Tx<RO>, path: &str, order: OrderArgs) -> Result<()> {
    if order.plain {
        println!("[1/1] Generating preimage file...");
        cmds::generate(
            path,
            PlainIterator::new(tx)?,
            AddressProgressBar::new(false),
        )?;
    } else if order.eip4762 {
        println!("[1/2] Ordering account addresses by hash...");
        let mut pb = AddressProgressBar::new(false);
        let it = Eip4762Iterator::new(tx, Some(|addr| pb.progress(addr)))?;
        println!("[2/2] Generating preimage file...");
        cmds::generate(path, it, AddressProgressBar::new(true))?;
    } else {
        return Err(anyhow!("No ordering specified"));
    }
    Ok(())
}

fn verify_cmd(tx: Tx<RO>, path: &str, order: OrderArgs) -> Result<()> {
    if order.plain {
        println!("[1/2] Verifying provided preimage file...");
        cmds::verify(
            path,
            PlainIterator::new(tx)?,
            AddressProgressBar::new(false),
        )?;
        println!("[2/2] The preimage file is valid!");
    } else if order.eip4762 {
        println!("[1/3] Ordering account addresses by hash...");
        let mut pb = AddressProgressBar::new(false);
        let it = Eip4762Iterator::new(tx, Some(|addr| pb.progress(addr)))?;
        println!("[2/3] Verifying provided preimage file...");
        cmds::verify(path, it, AddressProgressBar::new(true))?;
        println!("[3/3] The preimage file is valid!");
    } else {
        return Err(anyhow!("No ordering specified"));
    }
    Ok(())
}
