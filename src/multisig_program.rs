use std::{error::Error, sync::Arc};

use anchor_client::{Client, Program};
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer, system_program,
};
use squads_multisig::{
    client::{
        proposal_approve, proposal_create, vault_transaction_create, vault_transaction_execute,
    },
    pda::{get_proposal_pda, get_transaction_pda, get_vault_pda},
    vault_transaction::VaultTransactionMessageExt,
};
use squads_multisig_program::{Multisig, ProposalCreateArgs, ProposalVoteArgs, TransactionMessage};

pub struct MultisigProgram {
    program: Program<Arc<Keypair>>,
    multisig: Pubkey,
}

impl MultisigProgram {
    pub fn new(
        client: &Client<Arc<Keypair>>,
        program_id: Pubkey,
        multisig: Pubkey,
    ) -> Result<Self, Box<dyn Error>> {
        let program = client.program(program_id)?;
        Ok(MultisigProgram { program, multisig })
    }

    pub async fn get_multisig_account(&self) -> Result<Multisig, Box<dyn Error>> {
        let acc = self.program.account(self.multisig).await?;
        Ok(acc)
    }

    pub async fn get_current_transaction_index(&self) -> Result<u64, Box<dyn Error>> {
        Ok(self.get_multisig_account().await?.transaction_index)
    }

    pub async fn create_transaction_and_proposal(
        &self,
        proposer: &Keypair,
        payer: &Keypair,
        instructions: &[Instruction],
        transaction_index: u64,
    ) -> Result<(), Box<dyn Error>> {
        let vault_index = 0;

        let vault_pda = get_vault_pda(&self.multisig, vault_index, Some(&self.program.id())).0;
        let transaction_account_pda =
            get_transaction_pda(&self.multisig, transaction_index, Some(&self.program.id())).0;
        let proposal_pda =
            get_proposal_pda(&self.multisig, transaction_index, Some(&self.program.id())).0;

        let transaction_message = TransactionMessage::try_compile(&vault_pda, instructions, &[])?;

        let accounts = squads_multisig::client::VaultTransactionCreateAccounts {
            multisig: self.multisig,
            transaction: transaction_account_pda,
            system_program: system_program::ID,
            creator: proposer.pubkey(),
            rent_payer: payer.pubkey(),
        };

        let vault_tx_ix = vault_transaction_create(
            accounts,
            vault_index,
            0,
            &transaction_message,
            None,
            Some(self.program.id()),
        );

        let proposal_ix = proposal_create(
            squads_multisig::client::ProposalCreateAccounts {
                multisig: self.multisig,
                proposal: proposal_pda,
                creator: proposer.pubkey(),
                rent_payer: payer.pubkey(),
                system_program: system_program::ID,
            },
            ProposalCreateArgs {
                transaction_index,
                draft: false,
            },
            Some(self.program.id()),
        );

        let signature = self
            .program
            .request()
            .signer(&proposer)
            .signer(&payer)
            .instruction(vault_tx_ix)
            .instruction(proposal_ix)
            .send()
            .await?;

        println!("Signature: {}", signature);

        Ok(())
    }

    pub async fn approve_proposal(
        &self,
        approver: &Keypair,
        payer: &Keypair,
        transaction_index: u64,
    ) -> Result<(), Box<dyn Error>> {
        let proposal_pda =
            get_proposal_pda(&self.multisig, transaction_index, Some(&self.program.id())).0;

        println!("Proposal: {}", proposal_pda);

        let ix = proposal_approve(
            squads_multisig::client::ProposalVoteAccounts {
                multisig: self.multisig,
                member: approver.pubkey(),
                proposal: proposal_pda,
            },
            ProposalVoteArgs { memo: None },
            Some(self.program.id()),
        );

        let signature = self
            .program
            .request()
            .signer(&approver)
            .signer(&payer)
            .instruction(ix)
            .send()
            .await?;

        println!("Signature: {}", signature);

        Ok(())
    }

    pub async fn execute_transaction(
        &self,
        executor: &Keypair,
        payer: &Keypair,
        instruction_payer: &Keypair,
        transaction_index: u64,
        instructions: &[Instruction],
    ) -> Result<(), Box<dyn Error>> {
        let vault_index = 0;

        let vault_pda = get_vault_pda(&self.multisig, vault_index, Some(&self.program.id())).0;
        let proposal_pda =
            get_proposal_pda(&self.multisig, transaction_index, Some(&self.program.id())).0;
        let transaction_account_pda =
            get_transaction_pda(&self.multisig, transaction_index, Some(&self.program.id())).0;

        let message = TransactionMessage::try_compile(&vault_pda, instructions, &[])?;

        let ix = vault_transaction_execute(
            squads_multisig::client::VaultTransactionExecuteAccounts {
                multisig: self.multisig,
                proposal: proposal_pda,
                transaction: transaction_account_pda,
                member: executor.pubkey(),
            },
            vault_index,
            0,
            &message,
            &[],
            Some(self.program.id()),
        )?;

        let signature = self
            .program
            .request()
            .signer(&executor)
            .signer(&payer)
            .signer(&instruction_payer)
            .instruction(ix)
            .send()
            .await?;

        println!("Signature: {}", signature);

        Ok(())
    }
}
