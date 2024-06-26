use serde::{Deserialize, Serialize};

// From https://github.com/gemwalletcom/core/blob/main/crates/primitives/src/chain_type.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ChainType {
    Ethereum,
    Bitcoin,
    Solana,
    Cosmos,
    Ton,
    Tron,
    Aptos,
    Sui,
    Xrp,
    Near,
}
