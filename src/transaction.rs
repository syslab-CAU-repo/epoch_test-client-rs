use alloy::primitives::FixedBytes;
use jsonrpsee::core::traits::ToRpcParams;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

#[derive(Clone, Debug, Serialize)]
pub enum Transaction {
    EthRaw(EthRawTransaction),
    Raw(RawTransaction),
    Encrypted(String),
}

impl Transaction {
    pub fn eth_raw_transaction(encoded_transaction: String) -> Self {
        Self::EthRaw(encoded_transaction.into())
    }

    pub fn raw_transaction(rollup_id: String, encoded_transaction: String) -> Self {
        Self::Raw((rollup_id, encoded_transaction).into())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct EthRawTransaction(Vec<String>);

impl From<String> for EthRawTransaction {
    fn from(value: String) -> Self {
        Self(vec![value])
    }
}

impl ToRpcParams for EthRawTransaction {
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, serde_json::Error> {
        let json = serde_json::to_string(&self)?;

        RawValue::from_string(json).map(Some)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RawTransaction {
    rollup_id: String,
    raw_transaction: RawTransactionInner,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
#[allow(unused)]
enum RawTransactionInner {
    Eth(String),
    EthBundle(String),
}

impl From<(String, String)> for RawTransaction {
    fn from(value: (String, String)) -> Self {
        Self {
            rollup_id: value.0,
            raw_transaction: RawTransactionInner::Eth(value.1),
        }
    }
}

impl ToRpcParams for RawTransaction {
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, serde_json::Error> {
        let json = serde_json::to_string(&self)?;

        RawValue::from_string(json).map(Some)
    }
}

#[derive(Clone, Debug)]
pub enum TransactionResponse {
    TransactionHash(FixedBytes<32>),
    OrderCommitment(OrderCommitment),
}

#[derive(Clone, Debug, Deserialize)]
pub struct OrderCommitment {
    pub data: OrderCommitmentData,
    pub signature: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OrderCommitmentData {
    pub rollup_id: String,
    pub block_height: u64,
    pub transaction_order: u64,
    pub pre_merkle_path: Vec<String>,
}
