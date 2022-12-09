use std::collections::HashSet;
use std::ops::{AddAssign, DivAssign};
use std::sync::Arc;
use std::time::Instant;

use lite_rpc_tests::{generate_txs, new_funded_payer};
use log::info;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_client::SerializableTransaction;
use solana_sdk::native_token::LAMPORTS_PER_SOL;

use lite_rpc_tests::client::{LiteClient, LOCAL_LIGHT_RPC_ADDR};
use simplelog::*;

const NUM_OF_TXS: usize = 10_000;
const NUM_OF_RUNS: usize = 2;

#[derive(Debug, Default)]
struct Metric {
    elapsed_sec: f64,
    tps: f64,
}

#[derive(Default)]
struct AvgMetric {
    num_of_runs: usize,
    total_metric: Metric,
}

impl AddAssign for Metric {
    fn add_assign(&mut self, rhs: Self) {
        self.elapsed_sec += rhs.elapsed_sec;
        self.tps += rhs.tps;
    }
}

impl DivAssign<f64> for Metric {
    fn div_assign(&mut self, rhs: f64) {
        self.elapsed_sec /= rhs;
        self.tps /= rhs;
    }
}

impl AddAssign<Metric> for AvgMetric {
    fn add_assign(&mut self, rhs: Metric) {
        self.num_of_runs += 1;
        self.total_metric += rhs;
    }
}

impl From<AvgMetric> for Metric {
    fn from(mut avg_metric: AvgMetric) -> Self {
        avg_metric.total_metric /= avg_metric.num_of_runs as f64;
        avg_metric.total_metric
    }
}

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
        let metric = foo(lite_client.clone()).await;
        info!("Run {run_num}: Sent and Confirmed {NUM_OF_TXS} tx(s) in {metric:?}");
        avg_metric += metric;
    }

    info!("Avg Metric {:?}", Metric::from(avg_metric));
}

async fn foo(lite_client: Arc<LiteClient>) -> Metric {
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

    Metric { elapsed_sec, tps }
}
