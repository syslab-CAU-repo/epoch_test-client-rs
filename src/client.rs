use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

pub type EthereumProvider = FillProvider<
    JoinFill<
        JoinFill<
            Identity,
            JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
        >,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider<Http<Client>>,
    Http<Client>,
    Ethereum,
>;

use alloy::{
    eips::eip2718::Encodable2718,
    network::{Ethereum, EthereumWallet, TransactionBuilder},
    providers::{
        fillers::{
            BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller,
            WalletFiller,
        },
        Identity, ProviderBuilder, RootProvider, WalletProvider,
    },
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
    transports::http::{reqwest::Url, Client, Http},
};
use jsonrpsee::{core::client::ClientT, http_client::HttpClient};
use tokio::{
    task::JoinHandle,
    time::{Duration, Instant},
};

use crate::transaction::SendRawTransaction;

pub struct Flag(Arc<AtomicBool>);

impl Clone for Flag {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Default for Flag {
    fn default() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }
}

impl Flag {
    pub fn stopped(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }

    pub fn stop(&self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

pub struct TestClient {
    provider: EthereumProvider,
    nonce: u64,
    rpc_client: HttpClient,
    statistics: Statistics,
}

impl TestClient {
    pub fn provider(&self) -> &EthereumProvider {
        &self.provider
    }

    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    pub async fn init<T, F>(
        rpc_url: impl AsRef<str>,
        signing_key: impl AsRef<str>,
        transaction_builder: T,
        flag: Flag,
    ) -> Result<JoinHandle<Self>, TestClientError>
    where
        T: Fn(&Self) -> F + Send + 'static,
        F: std::future::Future<Output = Result<String, Box<dyn std::error::Error>>> + Send,
    {
        let signer = PrivateKeySigner::from_str(signing_key.as_ref())
            .map_err(TestClientError::LocalSigner)?;
        let wallet = EthereumWallet::from(signer);
        let url = Url::from_str(rpc_url.as_ref())
            .map_err(|error| TestClientError::ParseUrl(error.into()))?;
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(url);
        let rpc_client = HttpClient::builder()
            .request_timeout(Duration::from_millis(500))
            .build(rpc_url)
            .map_err(TestClientError::BuildRpcClient)?;

        let mut client = Self {
            provider,
            nonce: 0,
            rpc_client,
            statistics: Statistics::default(),
        };

        let handle = tokio::spawn(async move {
            while flag.stopped() {
                let time_start = Instant::now();
                let transaction = transaction_builder(&client).await.unwrap();
                // match client.send(transaction).await {
                //     Ok(_) => client.statistics.ok(),
                //     Err(_) => client.statistics.err(),
                // }
                let response_time_ms = time_start.elapsed().as_millis();
                client.nonce += 1;
                client.statistics.response_time_ms(response_time_ms);
            }

            client
        });

        Ok(handle)
    }

    /// Encode a given transaction into hexadecimal String.
    pub async fn encode_transaction(
        &self,
        transaction: TransactionRequest,
        encryption: bool,
    ) -> Result<EncodedTransaction, TestClientError> {
        let envelope = transaction
            .build(&self.provider.wallet())
            .await
            .map_err(TestClientError::EncodeTransaction)?;

        Ok(const_hex::encode_prefixed(envelope.encoded_2718()))
    }

    pub async fn send_raw_transaction(&self, parameter: &str) -> Result<(), TestClientError> {
        let parameter = SendRawTransaction {
            rollup_id: "".to_owned(),
            raw_transaction: parameter.to_owned(),
        };

        let a = self
            .rpc_client
            .request("send_raw_transaction", parameter)
            .await
            .map_err(TestClientError::Send)?;

        Ok(())
    }

    /// TODO:
    pub async fn send_encrypted_transaction(&self) -> Result<(), TestClientError> {
        // Implement SKDE encryption logic.
        Ok(())
    }
}

#[derive(Debug)]
pub enum TestClientError {
    LocalSigner(alloy::signers::local::LocalSignerError),
    ParseUrl(Box<dyn std::error::Error>),
    BuildRpcClient(jsonrpsee::core::ClientError),
    ParseAddress(const_hex::FromHexError),
    EncodeTransaction(alloy::network::TransactionBuilderError<Ethereum>),
    Send(jsonrpsee::core::ClientError),
}

impl std::fmt::Display for TestClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for TestClientError {}

#[derive(Clone, Debug)]
pub struct Statistics {
    success: u64,
    failure: u64,
    total: u64,
    response_time_ms: Vec<u128>,
}

impl Default for Statistics {
    fn default() -> Self {
        Self {
            success: 0,
            failure: 0,
            total: 0,
            response_time_ms: vec![],
        }
    }
}

impl Statistics {
    pub fn ok(&mut self) {
        self.success += 1;
        self.total += 1;
    }

    pub fn err(&mut self) {
        self.failure += 1;
        self.total += 1;
    }

    pub fn response_time_ms(&mut self, response_time_ms: u128) {
        self.response_time_ms.push(response_time_ms);
    }
}
