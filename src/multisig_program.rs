use std::{error::Error, sync::Arc};

use anchor_client::{Client, Program};
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer, system_program,
};
use squads_multisig::{
    client::{proposal_create, vault_transaction_create, VaultTransactionCreateAccounts},
    pda::{get_proposal_pda, get_transaction_pda, get_vault_pda},
    vault_transaction::VaultTransactionMessageExt,
};
use squads_multisig_program::{Multisig, ProposalCreateArgs, TransactionMessage};

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

    pub async fn create_transaction(
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

        let transaction_message = TransactionMessage::try_compile(&vault_pda, instructions, &[])?;

        let accounts = VaultTransactionCreateAccounts {
            multisig: self.multisig,
            transaction: transaction_account_pda,
            system_program: system_program::ID,
            creator: proposer.pubkey(),
            rent_payer: payer.pubkey(),
        };

        let ix = vault_transaction_create(
            accounts,
            vault_index,
            0,
            &transaction_message,
            None,
            Some(self.program.id()),
        );

        let signature = self
            .program
            .request()
            .signer(&proposer)
            .signer(&payer)
            .instruction(ix)
            .send()
            .await?;

        println!("Signature: {}", signature);

        Ok(())
    }

    pub async fn create_proposal(
        &self,
        proposer: &Keypair,
        payer: &Keypair,
        transaction_index: u64,
    ) -> Result<(), Box<dyn Error>> {
        let proposal =
            get_proposal_pda(&self.multisig, transaction_index, Some(&self.program.id())).0;

        println!("Proposal: {}", proposal);

        let ix = proposal_create(
            squads_multisig::client::ProposalCreateAccounts {
                multisig: self.multisig,
                proposal,
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
            .instruction(ix)
            .send()
            .await?;

        println!("Signature: {}", signature);

        Ok(())
    }
}
