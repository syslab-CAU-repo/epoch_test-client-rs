use std::time::Duration;

use jsonrpsee::http_client::HttpClient;

use crate::transaction::TransactionType;

pub struct TestClient {
    client: HttpClient,
}

impl std::fmt::Debug for TestClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.client)
    }
}

impl TestClient {
    pub fn new(rpc_url: impl AsRef<str>, request_timeout: u64) -> Result<Self, TestClientError> {
        Ok(Self {
            client: HttpClient::builder()
                .request_timeout(Duration::from_secs(request_timeout))
                .build(rpc_url)
                .map_err(TestClientError::BuildClient)?,
        })
    }

    #[inline(always)]
    pub async fn send_transaction(
        &self,
        transaction: TransactionType,
    ) -> Result<(), TestClientError> {
        Ok(())
    }

    #[inline(always)]
    async fn send_raw_transaction(&self) -> Result<(), TestClientError> {
        Ok(())
    }

    #[inline(always)]
    async fn send_encrypted_transaction(&self) -> Result<(), TestClientError> {
        Ok(())
    }
}

#[derive(Debug)]
pub enum TestClientError {
    BuildClient(jsonrpsee::core::ClientError),
    SendTransaction(jsonrpsee::http_client::transport::Error),
}
