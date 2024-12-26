use std::{sync::Arc, time::Duration};

use alloy::primitives::FixedBytes;
use jsonrpsee::{core::client::ClientT, http_client::HttpClient};
use tokio::sync::{mpsc, Mutex};

use crate::transaction::{Transaction, TransactionResponse};

pub type Sender = mpsc::Sender<Transaction>;
pub type Receiver = Arc<Mutex<mpsc::Receiver<Transaction>>>;

pub fn connection_channel(size: usize) -> (Sender, Receiver) {
    let (sender, receiver) = mpsc::channel(size);
    let receiver = Arc::new(Mutex::new(receiver));

    (sender, receiver)
}

pub struct Connection {
    rpc_client: HttpClient,
    total_requests: usize,
    responses: Vec<TransactionResponse>,
    receiver: Receiver,
}

impl Connection {
    pub fn new(
        rpc_url: impl AsRef<str>,
        request_timeout: u64,
        receiver: Receiver,
    ) -> Result<Self, ConnectionError> {
        let rpc_client = HttpClient::builder()
            .request_timeout(Duration::from_secs(request_timeout))
            .build(rpc_url)
            .map_err(ConnectionError::InitRpcClient)?;

        Ok(Self {
            rpc_client,
            total_requests: 0,
            responses: vec![],
            receiver,
        })
    }

    pub async fn init(mut self) -> Self {
        loop {
            if let Some(transaction) = self.receiver.lock().await.recv().await {
                match self.send_transaction(transaction).await {
                    Ok(transaction_response) => {
                        self.total_requests += 1;
                        self.responses.push(transaction_response);
                    }
                    Err(_error) => self.total_requests += 1,
                }
            } else {
                break;
            }
        }

        self
    }

    pub async fn send_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<TransactionResponse, ConnectionError> {
        match transaction {
            Transaction::EthRaw(_) => self.send_eth_raw_transaction(transaction).await,
            others => unimplemented!("Transaction type {:?} is unhandled.", others),
        }
    }

    pub async fn send_eth_raw_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<TransactionResponse, ConnectionError> {
        match self
            .rpc_client
            .request::<FixedBytes<32>, Transaction>("eth_sendRawTransaction", transaction)
            .await
        {
            Ok(transaction_hash) => Ok(transaction_hash.into()),
            Err(error) => Err(ConnectionError::Request(
                Method::SendEthRawTransaction,
                error,
            )),
        }
    }
}

#[derive(Debug)]
pub enum ConnectionError {
    InitRpcClient(jsonrpsee::core::ClientError),
    Request(Method, jsonrpsee::core::ClientError),
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ConnectionError {}

#[derive(Debug)]
pub enum Method {
    SendEthRawTransaction,
}
