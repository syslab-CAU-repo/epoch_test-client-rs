use std::{env, time::Duration};

use test_client_rs::{
    self,
    account::{Account, Accounts},
    config::Config,
    connection::{connection_channel, Connection, ConnectionError, Statistics},
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

    // Panics if the sum of generator and connection threads exceeds the number of
    // cores available.
    let total_threads = config.generator_threads() + config.connection_threads();
    let available_threads = std::thread::available_parallelism()
        .expect("Failed to get the available threads.")
        .get();
    if total_threads > available_threads {
        panic!(
            "The sum of generator and connection threads cannot exceed the available threads({}).",
            available_threads
        );
    }

    // Initialize the async runtime.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(total_threads)
        .max_blocking_threads(1)
        .build()?;

    // Initialize connections
    let (sender, receiver) = connection_channel(config.connections());
    let connections = (0..config.connections())
        .map(|_| Connection::new(config.rpc_url(), config.request_timeout(), receiver.clone()))
        .collect::<Result<Vec<Connection>, ConnectionError>>()?;
    let connection_handles: Vec<JoinHandle<Statistics>> = connections
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

    // Initialize generators.
    let flag = Flag::default();
    (0..config.generator_threads()).for_each(|_| {
        runtime.spawn(Generator::init(
            flag.clone(),
            transaction,
            accounts.clone(),
            sender.clone(),
        ));
    });
    tracing::info!("Initialized {} generator.", config.generator_threads());

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
    drop(sender);

    let mut statistics = runtime.block_on(async move {
        let mut aggregated = Statistics::default();

        for connection in connection_handles.into_iter() {
            let statistics = connection.await.unwrap();
            aggregated.total += statistics.total;
            aggregated.success += statistics.success;
            aggregated.failure += statistics.failure;
            aggregated.response_time.extend(statistics.response_time);
        }

        aggregated
    });

    let tps = statistics.tps(config.duration());
    let (p50, p90, p95, p99) = statistics.mean_response_time();

    tracing::info!(
        "Total: {}\nSuccess: {}\nFailure: {}\nTPS: {}\nLatency(ms):\n\tp50: {}\n\tp90: {}\n\tp95: {}\n\tp99: {}",
        statistics.total,
        statistics.success,
        statistics.failure,
        tps,
        p50, p90, p95, p99
    );

    Ok(())
}

#[inline(always)]
async fn transaction(accounts: Accounts) -> Transaction {
    use alloy::{
        eips::eip2718::Encodable2718, network::TransactionBuilder, primitives::U256,
        rpc::types::TransactionRequest,
    };
    use rand::seq::SliceRandom;

    let from = accounts.get(0).unwrap();
    let to = accounts[1..].choose(&mut rand::thread_rng()).unwrap();

    let transaction = TransactionRequest::default()
        .with_to(to.address())
        .with_nonce(from.fetch_add_nonce())
        .with_chain_id(67722)
        .with_value(U256::from(1))
        .with_gas_limit(21_000)
        .with_max_priority_fee_per_gas(1_000_000_000)
        .with_max_fee_per_gas(20_000_000_000);

    let envelope = transaction.build(from.wallet()).await.unwrap();
    let encoded_transaction = const_hex::encode_prefixed(envelope.encoded_2718());

    // Transaction::eth_raw_transaction(encoded_transaction)
    Transaction::raw_transaction(from.config().rollup_id().to_owned(), encoded_transaction)
}
