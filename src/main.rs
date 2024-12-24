use std::{
    env,
    sync::{atomic::AtomicBool, Arc},
};

use test_client_rs::{config::Config, connection::Statistics};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    let arguments: Vec<String> = env::args().skip(1).collect();
    let config_path = arguments
        .get(0)
        .expect("Provide the configuration file path.")
        .to_owned();
    let config = Config::open(&config_path)?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .max_blocking_threads(1)
        .worker_threads(config.threads())
        .build()?;

    let statistics = Statistics::default();
    let connections: Vec<tokio::task::JoinHandle<()>> = (0..config.connections())
        .map(|_index| {
            runtime.spawn({
                let config = config.clone();
                let statistics = statistics.clone();

                async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                    statistics.ok();
                }
            })
        })
        .collect();

    runtime.block_on(async move {
        for connection in connections {
            connection.await.unwrap();
        }
    });

    println!("{:?}", statistics);

    Ok(())
}

// async fn send_raw_transaction(§
//     test_client: &mut TestClient,
// ) -> Result<EncodedTransaction, TestClientError> {
//     let transaction = TransactionRequest::default()
//         .with_to(
//             "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
//                 .parse()
//                 .map_err(TestClientError::ParseAddress)?,
//         )
//         .with_nonce(test_client.nonce())
//         // .with_chain_id()
//         .with_value(U256::from(100))
//         .with_gas_limit(21_000)
//         .with_max_priority_fee_per_gas(1_000_000_000)
//         .with_max_fee_per_gas(20_000_000_000);

//     // let encoded_transaction =
//     test_client.raw_transaction(transaction).await?;

//     Ok(encoded_transaction)
// }

pub struct Flag {
    inner: Arc<FlagInner>,
}

struct FlagInner {
    started: AtomicBool,
    stopped: AtomicBool,
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
            inner: FlagInner {
                started: false.into(),
                stopped: false.into(),
            }
            .into(),
        }
    }
}
