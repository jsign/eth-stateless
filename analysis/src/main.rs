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
    let accounts_data = accounts::account_stats(&tx)?;
    {
        #[derive(Tabled)]
        struct AccountCounts {
            eoas: u64,
            contracts: u64,
            total: u64,
        }

        let table = Table::new(vec![AccountCounts {
            eoas: accounts_data.0,
            contracts: accounts_data.1,
            total: accounts_data.0 + accounts_data.1,
        }])
        .with(Style::markdown())
        .with(Panel::header("Accounts"))
        .to_string();

        println!("{}\n", table);
    }

    {
        let table = Table::new(vec![accounts_data.2])
            .with(Style::markdown())
            .with(Panel::header("Contracts code-length"))
            .to_string();

        println!("{}\n", table);
    }

    let storage_slots_data = accounts::storage_slots_stats(&tx)?;
    #[derive(Tabled)]
    struct StorageSlotStats {
        total: u64,
        average: u64,
        median: u64,
        p99: u64,
        max: u64,
    }
    {
        let table = Table::new(vec![StorageSlotStats {
            total: storage_slots_data.0,
            average: storage_slots_data.1.average,
            median: storage_slots_data.1.median,
            p99: storage_slots_data.1.p99,
            max: storage_slots_data.1.max,
        }])
        .with(Style::markdown())
        .with(Panel::header("Storage Slots stats"))
        .to_string();

        println!("{}\n", table);
    }

    Ok(())
}
