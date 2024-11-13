use std::{error::Error, sync::Arc};

use anchor_client::{Client, Cluster};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::pubkey;
use solana_sdk::{
    pubkey::Pubkey,
    signature::Keypair,
    signer::{EncodableKey, Signer},
};

const MULTISIG_PROGRAM_ID: Pubkey = pubkey!("GQGNGBWyWLQJHnnpxpNjd4qwqRK17Z3V6APS6ALee6KD");

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let payer: Arc<Keypair> = Arc::new(Keypair::read_from_file("./squads_payer.json")?);

    let members: Arc<[Keypair]> = Arc::new([
        Keypair::read_from_file("./squads_member1.json")?,
        Keypair::read_from_file("./squads_member2.json")?,
        Keypair::read_from_file("./squads_member3.json")?,
    ]);

    let cluster = Cluster::Localnet;

    let rpc_client = RpcClient::new(cluster.url().to_string());

    let client = Client::new(cluster, payer.clone());

    let spl_program = client.program(spl_memo::ID);

    let multisig_program = client.program(MULTISIG_PROGRAM_ID);

    let memo_str = "Hello, World!";

    let ix = spl_memo::build_memo(memo_str.as_bytes(), &[&payer.pubkey()]);

    println!("All good!");

    Ok(())
}
