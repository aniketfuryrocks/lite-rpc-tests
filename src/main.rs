use std::time::Duration;

use lite_rpc_tests::client::LiteClient;
use lite_rpc_tests::{create_transaction, new_funded_payer};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::Transaction;
use tokio::time::sleep;

const RPC_ADDR: &str = "http://127.0.0.1:8890";
//const RPC_ADDR: &str = "http://127.0.0.1:8899";
//const RPC_ADDR: &str = "http://5.62.126.197:10800";
const NUM_OF_TXS: usize = 100;

#[tokio::main]
async fn main() {
    let lite_client = LiteClient(RpcClient::new(RPC_ADDR.to_string()));
    let funded_payer = new_funded_payer(&lite_client, LAMPORTS_PER_SOL * 2).await;
    println!("payer {}", funded_payer.to_base58_string());

    let txs = generate_txs(&funded_payer, &lite_client.0).await;

    println!("sending tx(s)");

    for tx in txs {
        sleep(Duration::from_millis(800)).await;
        lite_client.send_transaction(&tx).await.unwrap();
        println!("tx {}", &tx.signatures[0]);
    }
}

async fn generate_txs(funded_payer: &Keypair, rpc_client: &RpcClient) -> Vec<Transaction> {
    let mut txs = Vec::with_capacity(NUM_OF_TXS);

    for _ in 0..NUM_OF_TXS {
        txs.push(create_transaction(funded_payer, rpc_client).await);
    }

    txs
}
