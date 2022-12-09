use std::{
    collections::HashSet,
    sync::Arc,
    time::{Duration, Instant},
};

use lite_rpc_tests::{
    generate_txs,
    metrics::{AvgMetric, Metric},
    new_funded_payer, LatestBlockHash,
};
use log::info;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_client::SerializableTransaction};
use solana_sdk::native_token::LAMPORTS_PER_SOL;

use lite_rpc_tests::client::{LiteClient, LOCAL_LIGHT_RPC_ADDR};
use simplelog::*;
use tokio::sync::{mpsc, RwLock};

const BENCH_TIME_LIMIT_IN_SECS: u64 = 60;
const NUM_OF_RUNS: usize = 1;

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
        info!(
            "Run {run_num}: Sent and Confirmed tx(s) in {metric:?} with tps {}",
            metric.calc_tps()
        );
        avg_metric += metric;
    }

    let avg_metric = Metric::from(avg_metric);

    info!(
        "Avg Metric {avg_metric:?} with tps {}",
        avg_metric.calc_tps()
    );
}

async fn foo(lite_client: Arc<LiteClient>, bench_time: Duration) -> Metric {
    let (metric_send, mut metric_recv) = mpsc::channel(1);

    let funded_payer = new_funded_payer(&lite_client, LAMPORTS_PER_SOL * 2000)
        .await
        .unwrap();

    let un_confirmed_txs: Arc<RwLock<HashSet<String>>> = Default::default();

    info!(
        "Sending and Confirming tx(s) in {} sec(s)",
        bench_time.as_secs_f64()
    );

    let send_fut = {
        let lite_client = lite_client.clone();
        let un_confirmed_txs = un_confirmed_txs.clone();
        tokio::spawn(async move {
            let start_time = Instant::now();

            while start_time.elapsed().as_secs() < bench_time.as_secs() {
                let txs = generate_txs(1, &lite_client.0, &funded_payer)
                    .await
                    .unwrap();

                let tx = &txs[0];

                lite_client.send_transaction(tx).await.unwrap();

                un_confirmed_txs
                    .write()
                    .await
                    .insert(tx.signatures[0].to_string());

                info!("Tx {}", &txs[0].signatures[0]);
            }

            info!("Sent tx(s)");
        })
    };

    let confirm_fut = tokio::spawn(async move {
        let start_time = Instant::now();
        let mut metric = Metric::default();

        while start_time.elapsed().as_secs() < bench_time.as_secs() {
            let mut confirmed_txs = Vec::new();

            {
                let un_confirmed_txs = un_confirmed_txs.read().await;

                for sig in un_confirmed_txs.iter() {
                    if lite_client.confirm_transaction(sig.clone()).await {
                        confirmed_txs.push(sig.clone());
                        metric.txs_confirmed += 1;
                        info!("Confirmed {sig}");
                    } else {
                        info!("Un confirmed {}", un_confirmed_txs.len());
                    }
                }
            }

            let mut un_confirmed_txs = un_confirmed_txs.write().await;

            for tx in confirmed_txs {
                un_confirmed_txs.remove(&tx);
            }
        }

        metric.time_elapsed = start_time.elapsed();
        metric.txs_un_confirmed = un_confirmed_txs.read().await.len() as u64;
        metric.txs_sent = metric.txs_confirmed + metric.txs_un_confirmed;

        metric_send.send(metric).await.unwrap();
    });

    let (res1, res2) = tokio::join!(send_fut, confirm_fut);
    res1.unwrap();
    res2.unwrap();

    metric_recv.recv().await.unwrap()
}
