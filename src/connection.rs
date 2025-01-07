use std::sync::Arc;

use alloy::primitives::FixedBytes;
use jsonrpsee::{core::client::ClientT, http_client::HttpClient};
use tokio::{
    sync::{mpsc, Mutex},
    time::{Duration, Instant},
};

use crate::{
    statistics::Statistics,
    transaction::{
        EthRawTransaction, OrderCommitment, RawTransaction, Transaction, TransactionResponse,
    },
};

pub type Sender = mpsc::Sender<Transaction>;
pub type Receiver = Arc<Mutex<mpsc::Receiver<Transaction>>>;

pub fn connection_channel(size: usize) -> (Sender, Receiver) {
    let (sender, receiver) = mpsc::channel(size);
    let receiver = Arc::new(Mutex::new(receiver));

    (sender, receiver)
}

pub struct Connection {
    statistics: Statistics,
    rpc_client: HttpClient,
    receiver: Receiver,
}

impl Connection {
    pub fn new(
        statistics: Statistics,
        rpc_url: impl AsRef<str>,
        request_timeout: u64,
        receiver: Receiver,
    ) -> Result<Self, ConnectionError> {
        let rpc_client = HttpClient::builder()
            .request_timeout(Duration::from_secs(request_timeout))
            .build(rpc_url)
            .map_err(ConnectionError::InitRpcClient)?;

        Ok(Self {
            statistics,
            rpc_client,
            receiver,
        })
    }

    pub async fn init(self) {
        loop {
            let mut receiver = self.receiver.lock().await;
            if let Some(transaction) = receiver.recv().await {
                drop(receiver);

                let time_start = Instant::now();
                self.statistics.sent().await;
                match self.send_transaction(transaction).await {
                    Ok(response) => {
                        let response_time = time_start.elapsed().as_millis();
                        self.statistics.succeed(response, response_time).await;
                    }
                    Err(error) => {
                        let response_time = time_start.elapsed().as_millis();
                        self.statistics.failed(error, response_time).await;
                    }
                }
            } else {
                break;
            }
        }
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
