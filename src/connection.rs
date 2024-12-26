use std::{sync::Arc, time::Duration};

use jsonrpsee::http_client::HttpClient;
use tokio::sync::{mpsc, Mutex};

use crate::transaction::{Transaction, TransactionResult};

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
            receiver,
        })
    }

    pub async fn init(self) -> Self {
        loop {
            if let Some(transaction) = self.receiver.lock().await.recv().await {
                self.send_transaction(transaction).await;
            } else {
                break;
            }
        }

        self
    }

    pub async fn send_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<TransactionResult, ConnectionError> {
        match transaction {
            Transaction::EthRaw(_) => self.send_eth_raw_transaction(transaction).await,
            others => unimplemented!("Transaction type {:?} is unhandled.", others),
        }
    }

    pub async fn send_eth_raw_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<TransactionResult, ConnectionError> {
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
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ConnectionError {}

// use std::{
//     future::Future,
//     sync::{
//         atomic::{AtomicBool, Ordering},
//         Arc,
//     },
//     time::Duration,
// };

// use alloy::primitives::FixedBytes;
// use jsonrpsee::{core::client::ClientT, http_client::HttpClient};
// use tokio::{
//     runtime::{Builder, Runtime},
//     sync::{mpsc, Mutex},
//     task::JoinHandle,
//     time::interval,
// };

// use crate::{
//     account::Account,
//     config::Config,
//     transaction::{Transaction, TransactionResult},
// };

// type Sender = mpsc::Sender<Transaction>;
// type Receiver = Arc<Mutex<mpsc::Receiver<Transaction>>>;

// pub fn init<T, F>(
//     config: &Config,
//     runtime: Runtime,
//     transaction_generator: T,
// ) -> Result<(), ConnectionError>
// where
//     T: Fn(Account) -> F + Send + 'static,
//     F: Future<Output = Transaction> + Send + 'static,
// {
//     // let accounts = runtime
//     //     .block_on(Account::from_config(&config))
//     //     .map_err(ConnectionError::InitAccounts)?;

//     // let (sender, receiver) = channel(config.signing_keys().len());

//     // let connection_handles

//     // let connection_handles = (0..config.connections())
//     //     .map(|_| Connection::init(receiver.clone()))
//     //     .collect();

//     // let flag = Flag::default();

//     // runtime.spawn({
//     //     let config = config.clone();
//     //     let flag = flag.clone();

//     //     async move {
//     //         while !flag.stopped() {
//     //             let transaction = transaction_generator().await;
//     //             let _ = sender.send(transaction).await;
//     //         }
//     //     }
//     // });

//     // runtime.spawn({
//     //     let duration = config.duration();
//     //     let flag = flag.clone();

//     //     async move {
//     //         let mut interval = interval(Duration::from_secs(1));
//     //         interval.tick().await;

//     //         for second in 0..duration {
//     //             interval.tick().await;
//     //             tracing::info!("{} seconds passed..", second + 1);
//     //         }

//     //         flag.stop();
//     //     }
//     // });

//     // runtime.block_on(async move {
//     //     for handle in connection_handles {
//     //         let connection = handle.await.unwrap();
//     //         tracing::warn!("{:?}", connection);
//     //     }
//     // });

//     Ok(())
// }

// pub struct Flag(Arc<AtomicBool>);

// impl Clone for Flag {
//     fn clone(&self) -> Self {
//         Self(self.0.clone())
//     }
// }

// impl Default for Flag {
//     fn default() -> Self {
//         Self(Arc::new(AtomicBool::new(false)))
//     }
// }

// impl Flag {
//     pub fn stopped(&self) -> bool {
//         self.0.load(Ordering::SeqCst)
//     }

//     pub fn stop(&self) {
//         self.0.store(true, Ordering::SeqCst)
//     }
// }

// pub struct Connection {
//     rpc_client: HttpClient,
//     total_requests: u64,
// }

// impl Connection {
//     pub fn new(rpc_url: impl AsRef<str>, request_timeout: u64) ->
// Result<Self, ConnectionError> {         let rpc_client =
// HttpClient::builder()
// .request_timeout(Duration::from_secs(request_timeout))
// .build(rpc_url)             .map_err(ConnectionError::InitRpcClient)?;

//         Ok(Self {
//             rpc_client,
//             total_requests: 0,
//         })
//     }

//     pub fn init(self, receiver: Receiver) -> JoinHandle<Self> {
//         tokio::spawn(async move {
//             loop {
//                 if let Some(transaction) = receiver.lock().await.recv().await
// {                     self.send_transaction(transaction).await;
//                 } else {
//                     break;
//                 }
//             }

//             self
//         })
//     }

//     pub async fn send_transaction(
//         &self,
//         transaction: Transaction,
//     ) -> Result<TransactionResult, ConnectionError> {
//         match transaction {
//             Transaction::EthRaw(_) =>
// self.send_eth_raw_transaction(transaction).await,             others =>
// unimplemented!("Transaction type {:?} is unhandled.", others),         }
//     }

//     pub async fn send_eth_raw_transaction(
//         &self,
//         transaction: Transaction,
//     ) -> Result<TransactionResult, ConnectionError> {
//         match self
//             .rpc_client
//             .request::<FixedBytes<32>, Transaction>("eth_sendRawTransaction",
// transaction)             .await
//         {
//             Ok(transaction_hash) => Ok(transaction_hash.into()),
//             Err(error) => Err(ConnectionError::Request(
//                 Method::SendEthRawTransaction,
//                 error,
//             )),
//         }
//     }
// }

// // pub struct Connection {
// //     client: HttpClient,
// //     total_requests: u64,
// //     transaction_hashes: Vec<TransactionResult>,
// // }

// // impl std::fmt::Debug for Connection {
// //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
// //         write!(f, "Total requests: {}", self.total_requests)
// //     }
// // }

// // impl Connection {
// //     pub fn new(rpc_url: impl AsRef<str>, request_timeout: u64) ->
// // Result<Self, ConnectionError> {         let client = HttpClient::builder()
// //             .request_timeout(Duration::from_secs(request_timeout))
// //             .build(rpc_url)
// //             .map_err(ConnectionError::Initialize)?;

// //         Ok(Self {
// //             client,
// //             total_requests: 0,
// //             transaction_hashes: vec![],
// //         })
// //     }

// //     pub fn init(
// //         mut self,
// //         runtime: &Runtime,
// //         queue: Arc<Mutex<Receiver<Transaction>>>,
// //     ) -> JoinHandle<Self> {
// //         runtime.spawn(async move {
// //             loop {
// //                 if let Some(transaction) = queue.lock().await.recv().await
// { //                     match self.send_transaction(transaction).await {
// //                         Ok(transaction_hash) => {
// //                             self.total_requests += 1;
// //
// self.transaction_hashes.push(transaction_hash); //                         }
// //                         Err(_error) => self.total_requests += 1,
// //                     }
// //                 } else {
// //                     return self;
// //                 }
// //             }
// //         })
// //     }
// // }

// #[derive(Debug)]
// pub enum ConnectionError {
//     Runtime(std::io::Error),
//     InitAccounts(crate::account::AccountError),
//     InitRpcClient(jsonrpsee::core::ClientError),
//     Request(jsonrpsee::core::ClientError),
// }

// impl std::fmt::Display for ConnectionError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }

// impl std::error::Error for ConnectionError {}
