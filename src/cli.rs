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
    #[command(about = "Creates a vault transaction")]
    CreateVaultTransaction {
        #[arg(help = "The message that is going to be memoed by spl_memo")]
        message: String,
    },
    #[command(about = "Creates a proposal")]
    CreateProposal {
        #[arg(help = "The transaction index that should be associated with this proposal")]
        transaction_index: u64,
    },
    #[command(about = "Approves a proposal")]
    ApproveProposal {
        #[arg(help = "The transaction index that is associated with this proposal")]
        transaction_index: u64,
        #[arg(help = "The index of the member that is doing the approval")]
        member_index: usize,
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
            Command::CreateProposal { transaction_index } => {
                multisig_program
                    .create_proposal(&members[0], rent_payer, transaction_index)
                    .await?
            }
            Command::ApproveProposal {
                transaction_index,
                member_index,
            } => {
                multisig_program
                    .approve_proposal(&members[member_index], rent_payer, transaction_index)
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
        .create_transaction(&members[0], rent_payer, &[ix], transaction_index)
        .await?;

    Ok(())
}
