use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JSONRPCRequest {
    pub id: i64,
    pub method: String,
    pub jsonrpc: String,
    pub params: Option<Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JSONRPCResponse<T> {
    pub result: T,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct XRPBlock {
    pub ledger_current_index: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NearBlock {
    pub header: NearBlockHeader,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NearBlockHeader {
    pub height: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TonBlock {
    pub consensus_block: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AptosBlock {
    pub block_height: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TronBlock {
    pub block_header: TronBlockHeader,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TronBlockHeader {
    pub raw_data: TronBlockHeaderRaw,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TronBlockHeaderRaw {
    pub number: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CosmosBlockResponse {
    pub block: CosmosBlock,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CosmosBlock {
    pub header: CosmosBlockHeader,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CosmosBlockHeader {
    pub height: String,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BitcoinBlock {
    pub blockbook: BitcoinBlockbook,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BitcoinBlockbook {
    #[serde(rename = "bestHeight")]
    pub best_height: u64,
}
