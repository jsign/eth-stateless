use anyhow::{anyhow, Result};
use clap::{command, Args, Parser};
use iterators::{eip7748::Eip7748Iterator, plain::PlainIterator};
use progress::AddressProgressBar;
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
        #[arg(long = "path", help = "Preimages file path to verify")]
        path: String,

        #[command(flatten)]
        order: OrderArgs,
    },

    #[command(
        name = "storage-slot-freq",
        about = "Analyze storage-slot 29-byte prefix frequency and size impact"
    )]
    StorageSlotsFrequency,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct OrderArgs {
    #[arg(long, help = "Use plain ordering")]
    plain: bool,
    #[arg(long, help = "Use EIP-7748 ordering (i.e: hashed)")]
    eip7748: bool,
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

    let tx = provider.tx_ref();
    match cli.subcmd {
        SubCommand::Generate { path, order } => generate_cmd(tx, &path, order)?,
        SubCommand::Verify { path, order } => {
            verify_cmd(tx, &path, order)?;
        }
        SubCommand::StorageSlotsFrequency => cmds::storage_slot_freq::<29>(tx, 1_000)?,
    }

    Ok(())
}

fn generate_cmd(tx: &Tx<RO>, path: &str, order: OrderArgs) -> Result<()> {
    if order.plain {
        println!("[1/1] Generating preimage file...");
        cmds::generate(
            path,
            PlainIterator::new(tx)?,
            AddressProgressBar::new(false),
        )?;
    } else if order.eip7748 {
        println!("[1/2] Ordering account addresses by hash...");
        let mut pb = AddressProgressBar::new(false);
        let it = Eip7748Iterator::new(tx, Some(|addr| pb.progress(addr)))?;
        println!("[2/2] Generating preimage file...");
        cmds::generate(path, it, AddressProgressBar::new(true))?;
    } else {
        return Err(anyhow!("No ordering specified"));
    }
    Ok(())
}

fn verify_cmd(tx: &Tx<RO>, path: &str, order: OrderArgs) -> Result<()> {
    if order.plain {
        println!("[1/2] Verifying provided preimage file...");
        cmds::verify(
            path,
            PlainIterator::new(tx)?,
            AddressProgressBar::new(false),
        )?;
        println!("[2/2] The preimage file is valid!");
    } else if order.eip7748 {
        println!("[1/3] Ordering account addresses by hash...");
        let mut pb = AddressProgressBar::new(false);
        let it = Eip7748Iterator::new(tx, Some(|addr| pb.progress(addr)))?;
        println!("[2/3] Verifying provided preimage file...");
        cmds::verify(path, it, AddressProgressBar::new(true))?;
        println!("[3/3] The preimage file is valid!");
    } else {
        return Err(anyhow!("No ordering specified"));
    }
    Ok(())
}
