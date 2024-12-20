use std::{env, str::FromStr};

use alloy::{
    eips::eip2718::Encodable2718,
    network::{EthereumWallet, TransactionBuilder},
    primitives::U256,
    providers::{Provider, ProviderBuilder, WalletProvider},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
};
use test_client_rs::{
    client::{TestClient, TestClientError},
    transaction::EncodedTransaction,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    let arguments: Vec<String> = env::args().skip(1).collect();
    let signing_key = arguments.get(0);

    Ok(())
}

async fn send_raw_transaction(
    test_client: &mut TestClient,
) -> Result<EncodedTransaction, TestClientError> {
    let transaction = TransactionRequest::default()
        .with_to(
            "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
                .parse()
                .map_err(TestClientError::ParseAddress)?,
        )
        .with_nonce(test_client.nonce())
        // .with_chain_id()
        .with_value(U256::from(100))
        .with_gas_limit(21_000)
        .with_max_priority_fee_per_gas(1_000_000_000)
        .with_max_fee_per_gas(20_000_000_000);

    // let encoded_transaction = test_client.raw_transaction(transaction).await?;

    Ok(encoded_transaction)
}
