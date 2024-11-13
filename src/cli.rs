use std::error::Error;

use clap::{Parser, Subcommand};
use solana_sdk::{signature::Keypair, signer::Signer};

use crate::multisig_program::MultisigProgram;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[command()]
    CreateVaultTransaction {
        #[arg()]
        message: String,
    },
}

impl Cli {
    pub async fn dispatch(
        &self,
        multisig_program: &MultisigProgram,
        members: &[Keypair],
        rent_payer: &Keypair,
        instruction_payer: &Keypair,
    ) -> Result<(), Box<dyn Error>> {
        match self.command {
            Command::CreateVaultTransaction { ref message } => {
                create_transaction(
                    multisig_program,
                    members,
                    rent_payer,
                    instruction_payer,
                    message,
                )
                .await?
            }
        }

        Ok(())
    }
}

async fn create_transaction(
    multisig_program: &MultisigProgram,
    members: &[Keypair],
    rent_payer: &Keypair,
    instruction_payer: &Keypair,
    message: &str,
) -> Result<(), Box<dyn Error>> {
    let ix = spl_memo::build_memo(message.as_bytes(), &[&instruction_payer.pubkey()]);

    let transaction_index = multisig_program.get_current_transaction_index().await? + 1;

    println!("Creating transaction");
    multisig_program
        .create_transaction(
            members.get(0).unwrap(),
            rent_payer,
            &[ix],
            transaction_index,
        )
        .await?;

    Ok(())
}
