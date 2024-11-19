use std::{error::Error, str::FromStr};

use clap::{Parser, Subcommand};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signature},
    signer::Signer,
};

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
    #[command(about = "Approves a proposal")]
    ApproveProposal {
        #[arg(help = "The transaction index that is associated with this proposal")]
        transaction_index: u64,
        #[arg(help = "The index of the member that is doing the approval")]
        member_index: usize,
    },
    #[command(about = "Executes a transaction")]
    ExecuteVaultTransaction {
        #[arg(help = "The message that is going to be memoed by spl_memo")]
        message: String,
        #[arg(help = "The transaction index that should be executed")]
        transaction_index: u64,
    },
    #[command(about = "Checks the memos from the instruction payer")]
    CheckMemos,
    #[command(about = "Gets the last transaction index")]
    TransactionIndex,
}

impl Cli {
    pub async fn dispatch(
        &self,
        multisig_program: &MultisigProgram,
        members: &[Keypair],
        rent_payer: &Keypair,
        instruction_payer: &Keypair,
        rpc_client: &RpcClient,
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
            Command::ApproveProposal {
                transaction_index,
                member_index,
            } => {
                multisig_program
                    .approve_proposal(&members[member_index], rent_payer, transaction_index)
                    .await?
            }
            Command::ExecuteVaultTransaction {
                ref message,
                transaction_index,
            } => {
                execute_transaction(
                    multisig_program,
                    members,
                    rent_payer,
                    instruction_payer,
                    message,
                    transaction_index,
                )
                .await?;
            }
            Command::CheckMemos => {
                let instruction_payer_pubkey = instruction_payer.pubkey();
                let signatures = rpc_client
                    .get_signatures_for_address(&instruction_payer_pubkey)
                    .await?;
                println!("Signatures: {:?}", signatures);

                println!("Signatures length: {}", signatures.len());
                for signature in signatures {
                    let tx = rpc_client
                        .get_transaction(
                            &Signature::from_str(&signature.signature)?,
                            solana_transaction_status::UiTransactionEncoding::Json,
                        )
                        .await?;
                    println!("Transaction: {:?}", tx);
                }
            }
            Command::TransactionIndex => {
                let transaction_index = multisig_program.get_current_transaction_index().await?;
                println!("Transaction index: {transaction_index}");
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
        .create_transaction_and_proposal(&members[0], rent_payer, &[ix], transaction_index)
        .await?;

    Ok(())
}

async fn execute_transaction(
    multisig_program: &MultisigProgram,
    members: &[Keypair],
    rent_payer: &Keypair,
    instruction_payer: &Keypair,
    message: &str,
    transaction_index: u64,
) -> Result<(), Box<dyn Error>> {
    let ix = spl_memo::build_memo(message.as_bytes(), &[&instruction_payer.pubkey()]);

    println!("Executing transaction");
    multisig_program
        .execute_transaction(
            &members[0],
            rent_payer,
            instruction_payer,
            transaction_index,
            &[ix],
        )
        .await?;

    Ok(())
}
