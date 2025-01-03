use std::sync::Arc;

use alloy::primitives::FixedBytes;
use jsonrpsee::{core::client::ClientT, http_client::HttpClient};
use tokio::{
    sync::{mpsc, Mutex},
    time::{Duration, Instant},
};

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

    pub async fn init(mut self) -> Statistics {
        loop {
            let mut receiver = self.receiver.lock().await;
            if let Some(transaction) = receiver.recv().await {
                drop(receiver);

                let time_start = Instant::now();
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
                let response_time = time_start.elapsed().as_millis();
                self.statistics.response_time.push(response_time);
            } else {
                break;
            }
        }

        self.statistics
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

#[derive(Default)]
pub struct Statistics {
    pub total: u64,
    pub success: u32,
    pub failure: u32,
    pub response_time: Vec<u128>,
}

impl Statistics {
    pub fn tps(&self, duration: u64) -> u64 {
        u64::from(self.success) / duration
    }

    /// # Panics
    ///
    /// The function panics if it fails to get the 'N'th index in the response
    /// time vector.
    ///
    /// Return 50th, 90th, 95th and 99th percentile values.
    pub fn mean_response_time(&mut self) -> (u128, u128, u128, u128) {
        self.response_time.sort();
        let k = self.response_time.len();

        let p50 = self.get_nth_response_time(k * 50 / 100);
        let p90 = self.get_nth_response_time(k * 90 / 100);
        let p95 = self.get_nth_response_time(k * 95 / 100);
        let p99 = self.get_nth_response_time(k * 99 / 100);

        (p50, p90, p95, p99)
    }

    fn get_nth_response_time(&self, index: usize) -> u128 {
        let p = self
            .response_time
            .get(index)
            .ok_or_else(|| panic!("Failed to get {}th index.", index))
            .unwrap();

        *p
    }
}
