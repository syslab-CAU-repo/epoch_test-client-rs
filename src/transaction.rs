use jsonrpsee::core::traits::ToRpcParams;
use serde::Serialize;
use serde_json::value::RawValue;

#[derive(Clone, Debug, Serialize)]
pub enum EncodedTransaction {
    Raw(String),
    Encrypted(String),
}

#[derive(Clone, Debug, Serialize)]
pub struct SendEncryptedTransaction {
    pub rollup_id: String,
    pub raw_transaction: String,
}

impl SendEncryptedTransaction {
    pub const METHOD_NAME: &str = "send_encrypted_transaction";
}

impl ToRpcParams for SendEncryptedTransaction {
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, serde_json::Error> {
        let json = serde_json::to_string(&self)?;
        RawValue::from_string(json).map(Some)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SendRawTransaction {
    pub rollup_id: String,
    pub raw_transaction: String,
}

impl SendRawTransaction {
    pub const METHOD_NAME: &str = "send_raw_transaction";
}

impl ToRpcParams for SendRawTransaction {
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, serde_json::Error> {
        let json = serde_json::to_string(&self)?;
        RawValue::from_string(json).map(Some)
    }
}
