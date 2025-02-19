use std::{
    io::{BufWriter, Write},
    path::Path,
};

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
        .with(Panel::header("Accounts"))
        .to_string();

        println!("{}\n", table);
    }

    {
        let table = Table::new(vec![accounts_data.2])
            .with(Panel::header("Code lengths"))
            .to_string();

        println!("{}\n", table);
    }

    let stem_stats = accounts::stem_stats(&tx, 256)?;
    {
        let total_stems = stem_stats
            .iter()
            .map(|a| 1 + a.ss_stems.len() + a.code_stems as usize)
            .sum::<usize>();

        #[derive(Tabled)]
        struct StemCountRow {
            name: &'static str,
            total: u64,
            #[tabled(rename = "%", format = "{:.2}%")]
            percentage: f64,
        }
        let contract_header_stems = stem_stats.len() as u64;
        let storage_slots_stems = stem_stats.iter().map(|a| a.ss_stems.len() as u64).sum();
        let code_chunks_stems = stem_stats.iter().map(|a| a.code_stems as u64).sum();
        let table = Table::new([
            StemCountRow {
                name: "Contract header stems",
                total: contract_header_stems,
                percentage: contract_header_stems as f64 / total_stems as f64 * 100.0,
            },
            StemCountRow {
                name: "Storage-slots stems",
                total: storage_slots_stems,
                percentage: storage_slots_stems as f64 / total_stems as f64 * 100.0,
            },
            StemCountRow {
                name: "Code-chunks stems",
                total: code_chunks_stems,
                percentage: code_chunks_stems as f64 / total_stems as f64 * 100.0,
            },
        ])
        .with(Panel::header("Stems type counts"))
        .with(Panel::footer(format!("Total = {}", total_stems)))
        .to_string();

        println!("{}\n", table);
    }

    {
        #[derive(Tabled)]
        struct ContractStemRow {
            name: &'static str,
            average: u64,
            median: u64,
            p99: u64,
            max: u64,
        }
        let stats = calculate_stats(
            &mut stem_stats
                .iter()
                .map(|a| a.account_stem)
                .collect::<Vec<_>>(),
        );
        let table = Table::new([ContractStemRow {
            name: "Contract header stems",
            average: stats.average,
            median: stats.median,
            p99: stats.p99,
            max: stats.max,
        }])
        .with(Panel::header("Stems type counts"))
        .to_string();

        println!("{}\n", table);
    }

    // {
    //     #[derive(Tabled)]
    //     struct StorageSlotsStemRow {
    //         name: &'static str,
    //         total: u64,
    //     }

    //     let table = Table::new([StorageSlotsStemRow {
    //         name: "Accounts storage-slot stems",
    //         total: stem_stats.len() as u64,
    //         stats: calculate_stats(
    //             &mut stem_stats
    //                 .iter()
    //                 .map(|a| a.account_stem)
    //                 .collect::<Vec<_>>(),
    //         ),
    //     }]);
    // }

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

fn calculate_stats<T>(data: &mut [T]) -> Stats
where
    T: Copy + Into<u64> + Ord,
{
    data.sort();
    let count = data.len() as u64;
    let sum: u64 = data.iter().map(|&x| x.into()).sum();
    let average = sum / count;
    let median = data[count as usize / 2].into();
    let p99 = data[(count as f64 * 0.99) as usize].into();
    let max = data.last().map_or(0, |&x| x.into());

    Stats {
        sum,
        average,
        median,
        p99,
        max,
    }
}
