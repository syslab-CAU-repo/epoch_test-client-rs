use std::{
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use tokio::{
    runtime::Runtime,
    task::JoinHandle,
    time::{interval, sleep, Duration},
};

use crate::{client::TestClient, config::Config, transaction::TransactionType};

pub struct TestManager {
    config: Config,
    runtime: Runtime,
    flag: Flag,
}

impl TestManager {
    pub fn new(config: Config) -> Result<Self, TestManagerError> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .max_blocking_threads(1)
            .worker_threads(config.io_thread())
            .build()
            .map_err(TestManagerError::BuildRuntime)?;

        Ok(Self {
            runtime,
            config,
            flag: Flag::default(),
        })
    }

    pub fn start<T, F>(&self, transaction_builder: T) -> Result<Statistics, TestManagerError>
    where
        T: Fn() -> F + Copy + Send + 'static,
        F: Future<Output = TransactionType> + Send + 'static,
    {
        let test_clients: Vec<JoinHandle<Statistics>> = self
            .config
            .accounts()
            .iter()
            .map(|_account| {
                // let account = account.to_owned();
                let flag = self.flag.clone();
                let test_client =
                    TestClient::new(self.config.rpc_url(), self.config.request_timeout()).unwrap();
                let mut statistics = Statistics::default();

                self.runtime.spawn(async move {
                    while !flag.started() {
                        sleep(Duration::from_micros(1)).await;
                    }

                    while flag.running() {
                        let transaction = transaction_builder().await;
                        match test_client.send_transaction(transaction).await {
                            Ok(_) => {
                                statistics.total += 1;
                                statistics.success += 1;
                            }
                            Err(_error) => {
                                statistics.total += 1;
                                statistics.failure += 1;
                            }
                        }

                        sleep(Duration::from_micros(1)).await;
                    }

                    statistics
                })
            })
            .collect();

        let report = self.runtime.spawn({
            let config = self.config.clone();
            let flag = self.flag.clone();

            async move {
                let mut interval = interval(Duration::from_secs(1));
                interval.tick().await;

                flag.start();
                tracing::info!("Starting the test manager.");

                for index in 0..config.test_duration() {
                    interval.tick().await;
                    tracing::info!("{} seconds passed..", index + 1);
                }

                flag.stop();

                let mut report = Statistics::default();
                for test_client in test_clients {
                    let statistics = test_client.await.unwrap();
                    report.total += statistics.total;
                    report.success += statistics.success;
                    report.failure += statistics.failure;
                }

                report
            }
        });

        let report = self
            .runtime
            .block_on(report)
            .map_err(TestManagerError::JoinClients)?;

        Ok(report)
    }
}

#[derive(Debug)]
pub enum TestManagerError {
    BuildRuntime(std::io::Error),
    JoinClients(tokio::task::JoinError),
}

impl std::fmt::Display for TestManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for TestManagerError {}

pub struct Flag {
    inner: Arc<FlagInner>,
}

struct FlagInner {
    started: AtomicBool,
    running: AtomicBool,
}

impl Clone for Flag {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Default for Flag {
    fn default() -> Self {
        Self {
            inner: Arc::new(FlagInner {
                started: false.into(),
                running: true.into(),
            }),
        }
    }
}

impl Flag {
    pub fn started(&self) -> bool {
        self.inner.started.load(Ordering::SeqCst)
    }

    pub fn running(&self) -> bool {
        self.inner.running.load(Ordering::SeqCst)
    }

    pub fn start(&self) {
        self.inner.started.store(true, Ordering::SeqCst);
    }

    pub fn stop(&self) {
        self.inner.running.store(false, Ordering::SeqCst);
    }
}

#[derive(Debug, Default)]
pub struct Statistics {
    pub total: u64,
    pub success: u64,
    pub failure: u64,
    pub response_time: Vec<i64>,
}
