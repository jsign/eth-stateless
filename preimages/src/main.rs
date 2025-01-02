use std::path::Path;

use anyhow::{Context, Result};
use clap::{command, Parser};
use reth_db::mdbx::{tx::Tx, DatabaseArguments};

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
            short = 'o',
            long = "output-path",
            help = "Preimages file output path",
            default_value = "preimages.bin"
        )]
        path: String,
    },
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
        SubCommand::Generate { path } => unimplemented!("generate"),
    }

    Ok(())
}
