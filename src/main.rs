mod multisig_program;

use std::{error::Error, fs::read_to_string, sync::Arc};

use anchor_client::{Client, Cluster};
use multisig_program::MultisigProgram;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::Keypair,
    signer::{EncodableKey, Signer},
};

#[serde_as]
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct SolanaPubkey(#[serde_as(as = "DisplayFromStr")] solana_sdk::pubkey::Pubkey);

impl From<SolanaPubkey> for Pubkey {
    fn from(value: SolanaPubkey) -> Self {
        value.0
    }
}

#[derive(Deserialize, Serialize)]
struct Config {
    pub multisig_program_id: SolanaPubkey,
    pub multisig: SolanaPubkey,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config_file = read_to_string("./config.toml")?;
    let config: Config = toml::from_str(&config_file)?;

    let instruction_payer = Arc::new(Keypair::read_from_file("./instruction_payer.json")?);

    let rent_payer: Arc<Keypair> = Arc::new(Keypair::read_from_file("./squads_payer.json")?);

    let members: Arc<[Keypair]> = Arc::new([
        Keypair::read_from_file("./squads_member1.json")?,
        Keypair::read_from_file("./squads_member2.json")?,
        Keypair::read_from_file("./squads_member3.json")?,
    ]);

    let cluster = Cluster::Localnet;

    let rpc_client = RpcClient::new(cluster.url().to_string());

    let client = Client::new(cluster, rent_payer.clone());

    let spl_program = client.program(spl_memo::ID)?;

    let multisig_program = MultisigProgram::new(
        &client,
        config.multisig_program_id.into(),
        config.multisig.into(),
    )?;

    let memo_str = "Hello, World!";

    let ix = spl_memo::build_memo(memo_str.as_bytes(), &[&instruction_payer.pubkey()]);

    let transaction_index = multisig_program.get_current_transaction_index().await? + 1;

    println!("Creating transaction");
    multisig_program
        .create_transaction(
            members.get(0).unwrap(),
            rent_payer.as_ref(),
            &[ix],
            transaction_index,
        )
        .await?;

    println!("All good!");

    Ok(())
}
