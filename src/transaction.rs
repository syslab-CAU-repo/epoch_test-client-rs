use jsonrpsee::core::traits::ToRpcParams;
use serde::Serialize;
use serde_json::value::RawValue;

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum Transaction {
    EthRaw(Vec<String>),
    Raw(String),
    Encrypted(String),
}

impl Transaction {
    pub fn eth_raw_transaction(encoded_transaction: String) -> Self {
        Self::EthRaw(vec![encoded_transaction])
    }
}

impl ToRpcParams for Transaction {
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, serde_json::Error> {
        let json = serde_json::to_string(&self)?;
        RawValue::from_string(json).map(Some)
    }
}
