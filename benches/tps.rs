use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use lite_rpc_tests::{generate_txs, new_funded_payer};
use log::info;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_client::SerializableTransaction;
use solana_sdk::native_token::LAMPORTS_PER_SOL;

use lite_rpc_tests::client::{LiteClient, LOCAL_LIGHT_RPC_ADDR};
use simplelog::*;

const NUM_OF_TXS: usize = 60_000;

#[tokio::main]
async fn main() {
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    let lite_client = Arc::new(LiteClient(RpcClient::new(LOCAL_LIGHT_RPC_ADDR.to_string())));
    foo(lite_client).await
}

async fn foo(lite_client: Arc<LiteClient>) {
    let funded_payer = new_funded_payer(&lite_client, LAMPORTS_PER_SOL * 2000)
        .await
        .unwrap();

    let txs = generate_txs(NUM_OF_TXS, &lite_client.0, &funded_payer)
        .await
        .unwrap();

    let mut un_confirmed_txs: HashSet<String> = HashSet::with_capacity(txs.len());
    for tx in &txs {
        un_confirmed_txs.insert(tx.get_signature().to_string());
    }

    let start_time = Instant::now();

    info!("Sending and Confirming {NUM_OF_TXS} tx(s)");

    let send_fut = {
        let lite_client = lite_client.clone();
        tokio::spawn(async move {
            for tx in txs {
                lite_client.send_transaction(&tx).await.unwrap();
                info!("Tx {}", &tx.signatures[0]);
            }
            info!("Sent {NUM_OF_TXS} tx(s)");
        })
    };

    let confirm_fut = tokio::spawn(async move {
        while !un_confirmed_txs.is_empty() {
            let mut confirmed_txs = Vec::new();

            for sig in &un_confirmed_txs {
                if lite_client.confirm_transaction(sig.clone()).await {
                    confirmed_txs.push(sig.clone());
                    info!("Confirmed {sig}");
                }
            }

            for tx in confirmed_txs {
                un_confirmed_txs.remove(&tx);
            }

            info!(
                "Confirmed {} tx(s) out of {NUM_OF_TXS}",
                NUM_OF_TXS - un_confirmed_txs.len()
            );
        }
    });

    let (res1, res2) = tokio::join!(send_fut, confirm_fut);
    res1.unwrap();
    res2.unwrap();

    let elapsed_sec = start_time.elapsed().as_secs_f64();
    let tps = (NUM_OF_TXS as f64) / elapsed_sec;

    info!("Sent and Confirmed {NUM_OF_TXS} tx(s) in {elapsed_sec} with TPS {tps}");
}
