use anyhow::{anyhow, Result};
use clap::Parser;
use reth_chainspec::ChainSpecBuilder;
use reth_db::{
    mdbx::{tx::Tx, DatabaseArguments, MaxReadTransactionDuration, RO},
    DatabaseEnv,
};
use reth_node_ethereum::EthereumNode;
use reth_node_types::NodeTypesWithDBAdapter;
use reth_provider::{providers::StaticFileProvider, ProviderFactory, StageCheckpointReader};
use reth_stages::StageId;
use std::{path::Path, sync::Arc};
use tabled::{settings::Panel, Table, Tabled};

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

    let db_path = Path::new(&cli.datadir).join("db");
    let db = reth_db::open_db_read_only(
        db_path.as_ref(),
        DatabaseArguments::default()
            .with_max_read_transaction_duration(Some(MaxReadTransactionDuration::Unbounded)),
    )
    .map_err(|err| anyhow!(err))?;

    let spec = ChainSpecBuilder::mainnet().build();
    let factory = ProviderFactory::<NodeTypesWithDBAdapter<EthereumNode, Arc<DatabaseEnv>>>::new(
        db.into(),
        spec.into(),
        StaticFileProvider::read_only(db_path.join("static_files"), true)?,
    );
    let provider = factory.provider()?;

    let latest_block_number = provider
        .get_stage_checkpoint(StageId::Finish)?
        .map(|ch| ch.block_number)
        .ok_or(anyhow!("No finish checkpoint"))?;
    println!("Database block number: {:?}", latest_block_number);

    let tx = provider.into_tx();

    match cli.subcmd {
        SubCommand::AccountsStats => account_stats(tx)?,
    }

    Ok(())
}

fn account_stats(tx: Tx<RO>) -> Result<()> {
    let stats = accounts::account_stats(&tx, 256)?;
    {
        #[derive(Tabled)]
        struct AccountCounts {
            eoas: usize,
            contracts: usize,
            total: usize,
        }
        let eoa_count = stats.iter().filter(|a| a.bytecode_len == 0).count();
        let table = Table::new(vec![AccountCounts {
            eoas: eoa_count,
            contracts: stats.len() - eoa_count,
            total: stats.len(),
        }])
        .with(Panel::header("Accounts"))
        .to_string();

        println!("{}\n", table);
    }

    {
        let mut code_lens: Vec<u64> = stats
            .iter()
            .filter(|a| a.bytecode_len > 0)
            .map(|a| a.bytecode_len as u64)
            .collect();
        let table = Table::new(vec![calculate_stats(&mut code_lens)])
            .with(Panel::header("Code length"))
            .to_string();

        println!("{}\n", table);
    }

    {
        let mut num_storage_slots: Vec<u64> = stats
            .iter()
            .filter(|a| a.bytecode_len > 0)
            .map(|a| a.num_storage_slots as u64)
            .collect();
        let table = Table::new(vec![calculate_stats(&mut num_storage_slots)])
            .with(Panel::header("Contract storage slots count"))
            .to_string();

        println!("{}\n", table);
    }

    {
        let total_stems = stats
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
        let contract_header_stems = stats.len() as u64;
        let storage_slots_stems = stats.iter().map(|a| a.ss_stems.len() as u64).sum();
        let code_chunks_stems = stats.iter().map(|a| a.code_stems as u64).sum();
        let table = Table::new([
            StemCountRow {
                name: "Accounts header stems",
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
        let account_stats =
            calculate_stats(&mut stats.iter().map(|a| a.account_stem).collect::<Vec<_>>());
        let ss_stats = calculate_stats(
            &mut stats
                .iter()
                .flat_map(|a| a.ss_stems.clone())
                .collect::<Vec<_>>(),
        );

        let table = Table::new([
            ContractStemRow {
                name: "Accounts header stems",
                average: account_stats.average,
                median: account_stats.median,
                p99: account_stats.p99,
                max: account_stats.max,
            },
            ContractStemRow {
                name: "Storage slots stems",
                average: ss_stats.average,
                median: ss_stats.median,
                p99: ss_stats.p99,
                max: ss_stats.max,
            },
        ])
        .with(Panel::header("Stems non-zero values count distribution"))
        .to_string();

        println!("{}\n", table);
    }

    {
        #[derive(Tabled)]
        struct SingleSlotStem {
            #[tabled(rename = "Storage-slot stems with single non-zero values")]
            single_slot_stems: usize,
        }
        let table = Table::new([SingleSlotStem {
            single_slot_stems: stats
                .iter()
                .map(|a| a.ss_stems.iter().filter(|s| **s == 1).count())
                .sum(),
        }])
        // .with(Panel::header("Single-slot stems"))
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
