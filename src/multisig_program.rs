use std::{error::Error, sync::Arc};

use anchor_client::{anchor_lang::AnchorSerialize, Client, Program};
use solana_sdk::{
    instruction::Instruction, message::Message, pubkey::Pubkey, signature::Keypair, signer::Signer,
    system_program,
};
use squads_multisig_program::{
    instruction::VaultTransactionCreate, Multisig, VaultTransactionCreateArgs,
};

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
        let (transaction_account_pda, _transaction_account_bump) = Pubkey::find_program_address(
            &[
                "multisig".as_bytes(),
                self.multisig.as_ref(),
                "transaction".as_bytes(),
                &transaction_index.to_le_bytes(),
            ],
            &self.program.id(),
        );
        let transaction_message = Message::new(instructions, Some(&payer.pubkey()));

        let create_transaction = VaultTransactionCreate {
            args: VaultTransactionCreateArgs {
                vault_index: 0,
                ephemeral_signers: 0,
                transaction_message: transaction_message.serialize().try_to_vec()?,
                memo: None,
            },
        };

        let accounts = squads_multisig_program::accounts::VaultTransactionCreate {
            multisig: self.multisig,
            transaction: transaction_account_pda,
            system_program: system_program::ID,
            creator: proposer.pubkey(),
            rent_payer: payer.pubkey(),
        };

        self.program
            .request()
            .signer(&proposer)
            .signer(&payer)
            .accounts(accounts)
            .args(create_transaction)
            .send()
            .await?;

        Ok(())
    }
}
