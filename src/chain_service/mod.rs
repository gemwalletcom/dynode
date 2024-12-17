mod model;

use bytes::{Buf, Bytes};
use http_body_util::{BodyExt, Empty, Full};
use hyper::{
    header::{self},
    Method, Request,
};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use model::{
    AptosBlock, BitcoinBlock, CosmosBlockResponse, JSONRPCRequest, JSONRPCResponse, NearBlock,
    TonBlock, TronBlock, XRPBlock,
};
use serde::de::DeserializeOwned;
use serde_json::Value;

use serde_json::to_vec;
use primitives::ChainType;

pub struct ChainService {
    pub chain_type: ChainType,
    pub url: String,
}

impl ChainService {
    pub async fn get_block_number(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        match self.chain_type {
            ChainType::Ethereum => {
                let block_hex = self
                    .get_json_rpc_data::<JSONRPCResponse<String>>("eth_blockNumber", None)
                    .await?
                    .result;
                Ok(u64::from_str_radix(&block_hex[2..], 16)?)
            }
            ChainType::Bitcoin => Ok(self
                .get_data::<BitcoinBlock>(Method::GET, "/api/")
                .await?
                .blockbook
                .best_height),
            ChainType::Solana => Ok(self
                .get_json_rpc_data::<JSONRPCResponse<u64>>("getSlot", None)
                .await?
                .result),
            ChainType::Cosmos => Ok(self
                .get_data::<CosmosBlockResponse>(
                    Method::GET,
                    "/cosmos/base/tendermint/v1beta1/blocks/latest",
                )
                .await?
                .block
                .header
                .height
                .parse::<u64>()
                .expect("number should be a u64")),
            ChainType::Ton => Ok(self
                .get_data::<JSONRPCResponse<TonBlock>>(Method::GET, "/api/v2/getConsensusBlock")
                .await?
                .result
                .consensus_block),
            ChainType::Tron => Ok(self
                .get_data::<TronBlock>(Method::POST, "/wallet/getnowblock")
                .await?
                .block_header
                .raw_data
                .number),
            ChainType::Aptos => Ok(self
                .get_data::<AptosBlock>(Method::GET, "/v1/")
                .await?
                .block_height
                .parse::<u64>()
                .expect("number should be a u64")),
            ChainType::Sui => {
                let block = self
                    .get_json_rpc_data::<JSONRPCResponse<String>>(
                        "sui_getLatestCheckpointSequenceNumber",
                        None,
                    )
                    .await?
                    .result
                    .parse::<u64>()
                    .expect("number should be a u64");
                Ok(block)
            }
            ChainType::Xrp => {
                let block = self
                    .get_json_rpc_data::<JSONRPCResponse<XRPBlock>>("ledger_current", None)
                    .await?
                    .result;
                Ok(block.ledger_current_index)
            }
            ChainType::Near => {
                let data = r#"{"finality": "final"}"#;
                let params: Value = serde_json::from_str(data)?;
                let block = self
                    .get_json_rpc_data::<JSONRPCResponse<NearBlock>>("block", Some(params))
                    .await?
                    .result
                    .header
                    .height;
                Ok(block)
            }
        }
    }

    pub async fn get_json_rpc_data<T: DeserializeOwned>(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        let client =
            Client::builder(hyper_util::rt::TokioExecutor::new()).build(HttpsConnector::new());
        let uri = self.url.parse::<hyper::Uri>().expect("invalid url");

        let payload = JSONRPCRequest {
            id: 1,
            method: method.to_string(),
            jsonrpc: "2.0".to_string(),
            params,
        };

        let json_bytes = to_vec(&payload)?;
        let body = Full::new(Bytes::from(json_bytes));

        let req = Request::builder()
            .method(Method::POST)
            .header(header::CONTENT_TYPE, "application/json")
            .uri(uri)
            .body(body)?;

        let res = client.request(req).await?;
        let body = res.collect().await?.to_bytes();

        Ok(serde_json::from_reader(body.reader())?)
    }

    pub async fn get_data<T: DeserializeOwned>(
        &self,
        mathod: Method,
        path: &str,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        let client =
            Client::builder(hyper_util::rt::TokioExecutor::new()).build(HttpsConnector::new());
        let uri = self.url.clone() + path;
        let uri = uri.parse::<hyper::Uri>().expect("invalid url");

        let req = Request::builder()
            .method(mathod)
            .header(header::CONTENT_TYPE, "application/json")
            .uri(uri)
            .body(Empty::<Bytes>::new())?;

        let res = client.request(req).await?;
        let body = res.collect().await?.to_bytes();

        Ok(serde_json::from_reader(body.reader())?)
    }
}
