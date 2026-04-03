use alloy::primitives::FixedBytes;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub enum Transaction {
    EthRaw(EthRawTransaction),
    Raw(RawTransaction),
    Encrypted(RawTransaction),
}

impl Transaction {
    pub fn eth_raw_transaction(encoded_transaction: String) -> Self {
        Self::EthRaw(EthRawTransaction::new(encoded_transaction))
    }

    pub fn raw_transaction(rollup_id: String, encoded_transaction: String) -> Self {
        Self::Raw(RawTransaction::new(rollup_id, encoded_transaction))
    }

    pub fn encrypted_transaction(rollup_id: String, encoded_transaction: String) -> Self {
        Self::Encrypted(RawTransaction::new(rollup_id, encoded_transaction))
    }

    /// Identifier for correlating send time with the serialized transaction payload.
    pub fn tx_key(&self) -> String {
        match self {
            Transaction::EthRaw(t) => t.tx_key(),
            Transaction::Raw(t) | Transaction::Encrypted(t) => t.tx_key(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct EthRawTransaction(Vec<String>);

impl EthRawTransaction {
    pub fn new(encoded_transaction: String) -> Self {
        Self(vec![encoded_transaction])
    }

    pub fn tx_key(&self) -> String {
        self.0[0].clone()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RawTransaction {
    rollup_id: String,
    raw_transaction: RawTransactionData,
}

#[derive(Clone, Debug, Serialize)]
pub struct RawTransactionData {
    #[serde(rename = "type")]
    transaction_type: String,
    data: String,
}

impl RawTransaction {
    pub fn new(rollup_id: String, encoded_transaction: String) -> Self {
        Self {
            rollup_id,
            raw_transaction: RawTransactionData {
                transaction_type: "eth".to_owned(),
                data: encoded_transaction,
            },
        }
    }

    pub fn tx_key(&self) -> String {
        self.raw_transaction.data.clone()
    }
}

#[derive(Clone, Debug, Deserialize)]

pub enum TransactionResponse {
    TransactionHash(FixedBytes<32>),
    OrderCommitment(serde_json::Value),
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
