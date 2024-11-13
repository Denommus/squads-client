use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use solana_sdk::pubkey::Pubkey;

#[serde_as]
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct SolanaPubkey(#[serde_as(as = "DisplayFromStr")] solana_sdk::pubkey::Pubkey);

impl From<SolanaPubkey> for Pubkey {
    fn from(value: SolanaPubkey) -> Self {
        value.0
    }
}

#[derive(Deserialize, Serialize)]
pub struct ClientConfig {
    pub multisig_program_id: SolanaPubkey,
    pub multisig: SolanaPubkey,
}
