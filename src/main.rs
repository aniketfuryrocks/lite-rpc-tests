use lite_rpc_tests::client::LiteClient;
use lite_rpc_tests::{generate_txs, new_funded_payer};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::native_token::LAMPORTS_PER_SOL;

const RPC_ADDR: &str = "http://127.0.0.1:8890";
const NUM_OF_TXS: usize = 1000;

#[tokio::main]
async fn main() {
    let lite_client = LiteClient(RpcClient::new(RPC_ADDR.to_string()));
    let funded_payer = new_funded_payer(&lite_client, LAMPORTS_PER_SOL * 2).await;

    println!("payer {}", funded_payer.to_base58_string());

    let txs = generate_txs(NUM_OF_TXS, &lite_client.0, &funded_payer).await;

    println!("sending tx(s)");

    for tx in txs {
        lite_client.send_transaction(&tx).await.unwrap();
        println!("tx {}", &tx.signatures[0]);
    }
}
