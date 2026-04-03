# test-client-rs

Rust client that hammers a Radius-style JSON-RPC endpoint with transactions for load testing. Transaction counts, connections, RPC URLs, and signing keys all come from a TOML file.

## Prerequisites

You need Rust (2021 edition). The build pulls `radius-sdk` from Git, so a blocked or offline network will fail the build.

## Build

```bash
cargo build --release
```

## Run

```bash
cargo run --release ./configs/default.toml
```

If `cargo` treats the config path as its own flag, pass it after `--`:

```bash
cargo run --release -- ./configs/default.toml
```

Arguments: first is the config file (required). Second is optional—output path for send timestamps. If you omit it, data goes to `send_time_map.jsonl`, one JSON object per line (`raw_transaction`, `send_epoch_ms`).

## Config (`configs/default.toml`)

| Field | Description |
|------|-------------|
| `total_transactions` | How many transactions to enqueue |
| `connection_threads` | Tokio worker threads; must not exceed logical cores or the binary panics |
| `connections` | Number of concurrent connections |
| `duration` | Run time in seconds; logs every second, then closes the send channel and tears down connections |
| `ethereum_rpc_url` | Ethereum-compatible RPC |
| `rpc_url` | Rollup / target JSON-RPC |
| `request_timeout` | RPC timeout in seconds |
| `chain_id` | Chain ID |
| `rollup_id` | Rollup identifier string |
| `signing_keys` | List of hex private keys (with `0x` prefix) |

Before you run it, sanity-check that `ethereum_rpc_url` and `rpc_url` actually point at live endpoints.
