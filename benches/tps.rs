use std::{
    collections::HashSet,
    sync::Arc,
    time::{Duration, Instant},
};

use lite_rpc_tests::{
    create_transaction,
    metrics::{AvgMetric, Metric},
    new_funded_payer, LatestBlockHash,
};
use log::info;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_client::SerializableTransaction};
use solana_sdk::native_token::LAMPORTS_PER_SOL;

use lite_rpc_tests::client::{LiteClient, LOCAL_LIGHT_RPC_ADDR};
use simplelog::*;
use tokio::sync::mpsc;

const BENCH_TIME_LIMIT_IN_SECS: u64 = 2;
const BLOCKHASH_TTL_SEC: u64 = 2;
const NUM_OF_RUNS: usize = 2;

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

    let mut avg_metric = AvgMetric::default();

    for run_num in 0..NUM_OF_RUNS {
        let metric = foo(
            lite_client.clone(),
            Duration::from_secs(BENCH_TIME_LIMIT_IN_SECS),
        )
        .await;
        info!("Run {run_num}: Sent and Confirmed tx(s) in {metric:?}");
        avg_metric += metric;
    }

    info!("Avg Metric {:?}", Metric::from(avg_metric));
}

async fn foo(lite_client: Arc<LiteClient>, bench_time: Duration) -> Metric {
    let (tx_send, mut tx_recv) = mpsc::unbounded_channel();
    let (metric_send, mut metric_recv) = mpsc::channel(1);
    let start_time = Arc::new(Instant::now());

    let send_fut = {
        let lite_client = lite_client.clone();

        tokio::spawn(async move {
            let funded_payer = new_funded_payer(&lite_client, LAMPORTS_PER_SOL * 2000)
                .await
                .unwrap();

            let latest_block_hash = lite_client.get_latest_blockhash().await.unwrap();

            //let mut latest_block_hash =
            //    LatestBlockHash::new(lite_client.clone(), Duration::from_secs(BLOCKHASH_TTL_SEC))
            //        .await
            //        .unwrap();

            let start_time = Instant::now();

            let mut tx_sent: usize = 0;

            while start_time.elapsed() < bench_time {
                let tx = create_transaction(&funded_payer, latest_block_hash);

                let sig = lite_client.send_transaction(&tx).await.unwrap();
                tx_send.send(sig.to_string()).unwrap();
                tx_sent += 1;
            }

            info!("Sent {tx_sent} in {} sec(s)", bench_time.as_secs_f64());

            drop(tx_send);
        })
    };

    let confirm_fut = tokio::spawn(async move {
        let mut un_confirmed_txs = HashSet::<String>::new();

        let mut metric = Metric::default();

        while let Some(sig) = tx_recv.recv().await {
            metric.txs_sent += 1;

            println!(
                "recv {sig} {} con {}",
                metric.txs_sent, metric.txs_confirmed
            );

            if !lite_client.confirm_transaction(sig.clone()).await {
                un_confirmed_txs.insert(sig);
            }

            //let mut confirmed_txs = Vec::new();

            //for sig in &un_confirmed_txs {
            //    if lite_client.confirm_transaction(sig.clone()).await {
            //        metric.txs_confirmed += 1;
            //        confirmed_txs.push(sig.clone());
            //        info!("Confirmed {sig}");
            //    }
            //}

            //for tx in confirmed_txs {
            //    un_confirmed_txs.remove(&tx);
            //}
        }

        println!("confirmed");

        metric.time_elapsed = start_time.elapsed();
        metric.txs_un_confirmed = un_confirmed_txs.len() as u64;

        metric_send.send(metric).await.unwrap();
    });

    let (res1, res2) = tokio::join!(send_fut, confirm_fut);
    res1.unwrap();
    res2.unwrap();

    metric_recv.recv().await.unwrap()
}
