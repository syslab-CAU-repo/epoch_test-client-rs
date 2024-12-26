use std::{env, time::Duration};

use test_client_rs::{
    self,
    account::{Account, Accounts},
    config::Config,
    connection::{connection_channel, Connection, ConnectionError},
    generator::{Flag, Generator},
    transaction::Transaction,
};
use tokio::{task::JoinHandle, time::interval};

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

    // Initialize connections
    let (sender, receiver) = connection_channel(config.connections());
    let connections = (0..config.connections())
        .map(|_| Connection::new(config.rpc_url(), config.request_timeout(), receiver.clone()))
        .collect::<Result<Vec<Connection>, ConnectionError>>()?;
    let connection_handles: Vec<JoinHandle<Connection>> = connections
        .into_iter()
        .map(|connection| runtime.spawn(connection.init()))
        .collect();
    tracing::info!("Initialized {} connections.", config.connections());

    // Initialize accounts from signing keys and set their nonces.
    let accounts = runtime.block_on(Account::from_config(&config))?;
    tracing::info!("Initialized {} accounts.", config.signing_keys().len());
    accounts.iter().for_each(|account| {
        tracing::info!(
            "Address: {:?}\t Nonce: {}",
            account.address(),
            account.nonce()
        )
    });

    // Initialize the generator.
    let flag = Flag::default();
    runtime.spawn(Generator::init(flag.clone(), transaction, accounts, sender));
    tracing::info!("Initialized the generator.");

    // Initialize the ticker.
    runtime.spawn({
        let duration = config.duration();
        let flag = flag.clone();

        async move {
            let mut interval = interval(Duration::from_secs(1));
            interval.tick().await;

            for second in 0..duration {
                interval.tick().await;
                tracing::info!("{} seconds passed..", second + 1);
            }

            flag.stop();
        }
    });

    runtime.block_on(async move {
        for connection in connection_handles.into_iter() {
            let _connection = connection.await.unwrap();
        }
    });

    Ok(())
}

async fn transaction(accounts: Accounts) -> Transaction {
    Transaction::EthRaw(vec!["0x".to_owned()])
}
