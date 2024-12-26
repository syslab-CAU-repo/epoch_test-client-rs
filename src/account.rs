use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use alloy::{
    network::{Ethereum, EthereumWallet},
    primitives::Address,
    providers::{
        fillers::{FillProvider, JoinFill, WalletFiller},
        Identity, ProviderBuilder, RootProvider, WalletProvider,
    },
    signers::local::PrivateKeySigner,
    transports::http::{Client, Http},
};
use jsonrpsee::{
    core::{client::ClientT, params::BatchRequestBuilder},
    http_client::HttpClient,
};

use crate::config::Config;

pub type Accounts = Arc<Vec<Account>>;

pub struct Account {
    inner: Arc<AccountInner>,
}

struct AccountInner {
    provider: FillProvider<
        JoinFill<Identity, WalletFiller<EthereumWallet>>,
        RootProvider<Http<Client>>,
        Http<Client>,
        Ethereum,
    >,
    nonce: AtomicUsize,
}

impl Clone for Account {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Account {
    pub async fn from_config(config: &Config) -> Result<Accounts, AccountError> {
        if config.signing_keys().is_empty() {
            return Err(AccountError::EmptySigningKey);
        }

        let accounts: Vec<Self> = config
            .signing_keys()
            .iter()
            .map(|signing_key| Self::new(signing_key, config.rpc_url()))
            .collect::<Result<Vec<Self>, AccountError>>()?;

        let rpc_client = HttpClient::builder()
            .request_timeout(Duration::from_secs(config.request_timeout()))
            .build(config.rpc_url())
            .map_err(AccountError::InitRpcClient)?;

        let mut batch_request = BatchRequestBuilder::new();
        accounts.iter().try_for_each(|account| {
            let parameter: Vec<String> = vec![account.address().to_string(), "latest".to_owned()];
            batch_request
                .insert("eth_getTransactionCount", parameter)
                .map_err(AccountError::BuildBatchRequest)
        })?;

        let batch_response = rpc_client
            .batch_request::<String>(batch_request)
            .await
            .map_err(AccountError::BatchRequest)?;

        for (response, account) in batch_response.into_iter().zip(accounts.iter()) {
            let nonce_string = response
                .map_err(|error| AccountError::BatchResponse(error.message().to_owned()))?;

            let nonce =
                usize::from_str_radix(&nonce_string[2..], 16).map_err(AccountError::ParseNonce)?;
            account.set_nonce(nonce);
        }

        Ok(Arc::new(accounts))
    }

    pub fn new(
        signing_key: impl AsRef<str>,
        rpc_url: impl AsRef<str>,
    ) -> Result<Self, AccountError> {
        let signer =
            PrivateKeySigner::from_str(signing_key.as_ref()).map_err(AccountError::Signer)?;
        let wallet = EthereumWallet::new(signer);
        let provider = ProviderBuilder::new()
            .wallet(wallet)
            .on_http(rpc_url.as_ref().parse().map_err(AccountError::Provider)?);

        let nonce = AtomicUsize::new(0);

        Ok(Self {
            inner: AccountInner { provider, nonce }.into(),
        })
    }

    pub fn address(&self) -> Address {
        self.inner.provider.wallet().default_signer().address()
    }

    fn set_nonce(&self, nonce: usize) {
        self.inner.nonce.store(nonce, Ordering::SeqCst);
    }

    pub fn nonce(&self) -> usize {
        self.inner.nonce.load(Ordering::SeqCst)
    }

    pub fn fetch_add_nonce(&self) -> usize {
        self.inner.nonce.fetch_add(1, Ordering::SeqCst)
    }
}

#[derive(Debug)]
pub enum AccountError {
    EmptySigningKey,
    ParseNonce(std::num::ParseIntError),
    Signer(alloy::signers::local::LocalSignerError),
    Provider(url::ParseError),
    InitRpcClient(jsonrpsee::core::ClientError),
    BuildBatchRequest(serde_json::Error),
    BatchRequest(jsonrpsee::core::ClientError),
    BatchResponse(String),
}

impl std::fmt::Display for AccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for AccountError {}
