use alloy::primitives::FixedBytes;
use jsonrpsee::core::traits::ToRpcParams;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
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
        Self::Raw(RawTransaction {
            rollup_id,
            raw_transaction: encoded_transaction,
        })
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
    raw_transaction: String,
}

impl ToRpcParams for RawTransaction {
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, serde_json::Error> {
        let json = serde_json::to_string(&self)?;

        RawValue::from_string(json).map(Some)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub enum TransactionResponse {
    TransactionHash(FixedBytes<32>),
    OrderCommitment(String),
}
