mod cli;
mod config;
mod multisig_program;

use std::{error::Error, fs::read_to_string, sync::Arc};

use anchor_client::{Client, Cluster};
use clap::Parser;
use cli::Cli;
use config::ClientConfig;
use multisig_program::MultisigProgram;
use solana_sdk::{signature::Keypair, signer::EncodableKey};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let config_file = read_to_string("./config.toml")?;
    let config: ClientConfig = toml::from_str(&config_file)?;

    let instruction_payer = Keypair::read_from_file("./instruction_payer.json")?;

    let rent_payer = Arc::new(Keypair::read_from_file("./squads_payer.json")?);

    let members = [
        Keypair::read_from_file("./squads_member1.json")?,
        Keypair::read_from_file("./squads_member2.json")?,
        Keypair::read_from_file("./squads_member3.json")?,
    ];

    let cluster = Cluster::Localnet;

    // let rpc_client = RpcClient::new(cluster.url().to_string());

    let client = Client::new(cluster, rent_payer.clone());

    let multisig_program = MultisigProgram::new(
        &client,
        config.multisig_program_id.into(),
        config.multisig.into(),
    )?;

    cli.dispatch(&multisig_program, &members, &rent_payer, &instruction_payer)
        .await?;

    println!("All good!");

    Ok(())
}
