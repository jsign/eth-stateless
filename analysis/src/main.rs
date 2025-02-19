use std::path::Path;

use anyhow::Result;
use clap::Parser;
use reth_db::mdbx::{tx::Tx, DatabaseArguments, RO};
use tabled::{
    settings::{Panel, Style},
    Table, Tabled,
};

mod accounts;

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
    #[command(name = "accounts-stats", about = "Generate account stats report")]
    AccountsStats,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let datadir = Path::new(&cli.datadir);

    let tx = {
        let db = reth_db::open_db_read_only(
            datadir.join("db").as_path(),
            DatabaseArguments::default().with_max_read_transaction_duration(Some(
                reth_db::mdbx::MaxReadTransactionDuration::Unbounded,
            )),
        )
        .unwrap();
        let tx = db.begin_ro_txn().unwrap();
        Tx::new(tx)
    };

    match cli.subcmd {
        SubCommand::AccountsStats => account_stats(tx)?,
    }

    Ok(())
}

fn account_stats(tx: Tx<RO>) -> Result<()> {
    #[derive(Tabled)]
    struct AccountCounts {
        eoas: u64,
        contracts: u64,
        total: u64,
    }

    let data = accounts::account_stats(&tx)?;

    let table = Table::new(vec![AccountCounts {
        eoas: data.0,
        contracts: data.1,
        total: data.0 + data.1,
    }])
    .with(Style::markdown())
    .with(Panel::header("Account type counts"))
    .to_string();

    println!("{}\n", table);

    let table = Table::new(vec![data.2])
        .with(Style::markdown())
        .with(Panel::header("Code length stats"))
        .to_string();

    println!("{}", table);

    Ok(())
}
