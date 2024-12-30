use std::{sync::Arc, time::Duration};

use alloy::primitives::FixedBytes;
use jsonrpsee::{core::client::ClientT, http_client::HttpClient};
use tokio::sync::{mpsc, Mutex};

use crate::transaction::{
    EthRawTransaction, OrderCommitment, RawTransaction, Transaction, TransactionResponse,
};

pub type Sender = mpsc::Sender<Transaction>;
pub type Receiver = Arc<Mutex<mpsc::Receiver<Transaction>>>;

pub fn connection_channel(size: usize) -> (Sender, Receiver) {
    let (sender, receiver) = mpsc::channel(size);
    let receiver = Arc::new(Mutex::new(receiver));

    (sender, receiver)
}

pub struct Connection {
    rpc_client: HttpClient,
    statistics: Statistics,
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
            statistics: Statistics::default(),
            responses: vec![],
            receiver,
        })
    }

    pub fn statistics(&self) -> &Statistics {
        &self.statistics
    }

    pub async fn init(mut self) -> Self {
        loop {
            let mut receiver = self.receiver.lock().await;
            if let Some(transaction) = receiver.recv().await {
                drop(receiver);

                match self.send_transaction(transaction).await {
                    Ok(transaction_response) => {
                        self.statistics.total += 1;
                        self.statistics.success += 1;
                        self.responses.push(transaction_response);
                    }
                    Err(error) => {
                        self.statistics.total += 1;
                        self.statistics.failure += 1;
                        tracing::error!("{:?}", error);
                    }
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
            Transaction::EthRaw(transaction) => self.send_eth_raw_transaction(transaction).await,
            Transaction::Raw(transaction) => self.send_raw_transaction(transaction).await,
            others => unimplemented!("Transaction type {:?} is unhandled.", others),
        }
    }

    pub async fn send_eth_raw_transaction(
        &self,
        transaction: EthRawTransaction,
    ) -> Result<TransactionResponse, ConnectionError> {
        match self
            .rpc_client
            .request::<FixedBytes<32>, EthRawTransaction>("eth_sendRawTransaction", transaction)
            .await
        {
            Ok(response) => Ok(TransactionResponse::TransactionHash(response)),
            Err(error) => Err(ConnectionError::Request(
                Method::SendEthRawTransaction,
                error,
            )),
        }
    }

    pub async fn send_raw_transaction(
        &self,
        transaction: RawTransaction,
    ) -> Result<TransactionResponse, ConnectionError> {
        match self
            .rpc_client
            .request::<OrderCommitment, RawTransaction>("send_raw_transaction", transaction)
            .await
        {
            Ok(response) => Ok(TransactionResponse::OrderCommitment(response)),
            Err(error) => Err(ConnectionError::Request(Method::SendRawTransaction, error)),
        }
    }
}

#[derive(Debug, Default)]
pub struct Statistics {
    pub total: u64,
    pub success: u32,
    pub failure: u32,
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
    SendRawTransaction,
}
