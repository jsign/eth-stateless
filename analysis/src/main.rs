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
    {
        let mut ss_counts = storage_slots_data
            .iter()
            .map(|a| a.total_slots)
            .collect::<Vec<_>>();

        let table = Table::new(vec![calculate_stats(&mut ss_counts)])
            .with(Style::markdown())
            .with(Panel::header("Storage Slots stats"))
            .to_string();

        println!("{}\n", table);
    }

    Ok(())
}

#[derive(Debug, Tabled)]
pub struct Stats {
    sum: u64,
    average: u64,
    median: u64,
    p99: u64,
    max: u64,
}

fn calculate_stats(data: &mut [u64]) -> Stats {
    data.sort();
    let count = data.len() as u64;
    let average = data.iter().sum::<u64>() / count;
    let median = data[count as usize / 2];
    let p99 = data[(count as f64 * 0.99) as usize];
    let max = *data.last().unwrap_or(&0);
    let sum = data.iter().sum();

    Stats {
        sum,
        average,
        median,
        p99,
        max,
    }
}
