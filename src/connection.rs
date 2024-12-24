use std::{
    sync::{
        atomic::{AtomicU32, AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use alloy::primitives::FixedBytes;
use jsonrpsee::{core::client::ClientT, http_client::HttpClient};

use crate::transaction::Transaction;

pub struct Connection {
    client: HttpClient,
}

impl Connection {
    pub fn init(rpc_url: impl AsRef<str>, request_timeout: u64) -> Result<Self, ConnectionError> {
        let client = HttpClient::builder()
            .request_timeout(Duration::from_secs(request_timeout))
            .build(rpc_url)
            .map_err(ConnectionError::Initialize)?;

        Ok(Self { client })
    }

    pub async fn send_transaction(&self, transaction: Transaction) -> Result<(), ConnectionError> {
        match transaction {
            Transaction::EthRaw(_) => self.send_eth_raw_transaction(transaction).await,
            others => unimplemented!("Transaction type {:?} is unhandled.", others),
        }
    }

    pub async fn send_eth_raw_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<(), ConnectionError> {
        match self
            .client
            .request::<FixedBytes<32>, Transaction>("eth_sendRawTransaction", transaction)
            .await
        {
            Ok(_) => Ok(()),
            Err(error) => Err(ConnectionError::Send(error)),
        }
    }
}

pub struct Statistics {
    inner: Arc<StatisticsInner>,
}

#[derive(Debug)]
struct StatisticsInner {
    total: AtomicU64,
    success: AtomicU32,
    failure: AtomicU32,
}

impl std::fmt::Debug for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl Clone for Statistics {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Default for Statistics {
    fn default() -> Self {
        Self {
            inner: StatisticsInner {
                total: 0.into(),
                success: 0.into(),
                failure: 0.into(),
            }
            .into(),
        }
    }
}

impl Statistics {
    pub fn ok(&self) {
        self.inner.total.fetch_add(1, Ordering::SeqCst);
        self.inner.success.fetch_add(1, Ordering::SeqCst);
    }

    pub fn err(&self) {
        self.inner.total.fetch_add(1, Ordering::SeqCst);
        self.inner.failure.fetch_add(1, Ordering::SeqCst);
    }
}

#[derive(Debug)]
pub enum ConnectionError {
    Initialize(jsonrpsee::core::ClientError),
    Send(jsonrpsee::core::ClientError),
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ConnectionError {}
