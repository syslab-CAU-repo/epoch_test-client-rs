use std::env;

use test_client_rs::{
    self,
    account::Account,
    config::Config,
    connection::{connection_channel, Connection, ConnectionError},
    transaction::Transaction,
};
use tokio::task::JoinHandle;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    let arguments: Vec<String> = env::args().skip(1).collect();
    let config_path = arguments
        .get(0)
        .expect("Provide the configuration file path.")
        .to_owned();
    let config = Config::open(&config_path)?;

    // Initialize the async runtime.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(config.threads())
        .max_blocking_threads(1)
        .build()?;

    // Initialize accounts from signing keys and set their nonces.
    let accounts = runtime.block_on(Account::from_config(&config))?;

    // Initialize connections
    let (sender, receiver) = connection_channel(accounts.len());
    let connections = (0..config.connections())
        .map(|_| Connection::new(config.rpc_url(), config.request_timeout(), receiver.clone()))
        .collect::<Result<Vec<Connection>, ConnectionError>>()?;
    let connection_handles: Vec<JoinHandle<Connection>> = connections
        .into_iter()
        .map(|connection| runtime.spawn(connection.init()))
        .collect();

    // TODO: Create a generator.

    runtime.block_on(async move {
        for connection in connection_handles.into_iter() {
            let _connection = connection.await.unwrap();
        }
    });

    Ok(())
}

async fn transaction(account: Account) -> Transaction {
    Transaction::EthRaw(vec!["0x".to_owned()])
}
