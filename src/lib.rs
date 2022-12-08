pub mod client;

use std::time::Duration;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::Signature;
use solana_sdk::{
    message::Message, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signature::Keypair,
    signer::Signer, system_instruction, transaction::Transaction,
};
use tokio::time::sleep;

use self::client::LiteClient;

pub async fn new_funded_payer(lite_client: &LiteClient, amount: u64) -> Keypair {
    let payer = Keypair::new();

    // request airdrop to payer
    let airdrop_sig = lite_client
        .request_airdrop(&payer.pubkey(), amount)
        .await
        .unwrap();

    println!("{airdrop_sig}");

    sleep(Duration::from_secs(20)).await;

    payer
}

pub async fn wait_till_confirmed(lite_client: &LiteClient, sig: &Signature) {
    while lite_client.confirm_transaction(sig.to_string()).await {}
}

pub async fn create_transaction(funded_payer: &Keypair, rpc_client: &RpcClient) -> Transaction {
    let to_pubkey = Pubkey::new_unique();

    // transfer instruction
    let instruction =
        system_instruction::transfer(&funded_payer.pubkey(), &to_pubkey, 1_000_000);

    let message = Message::new(&[instruction], Some(&funded_payer.pubkey()));

    let blockhash = rpc_client.get_latest_blockhash().await.unwrap();

    Transaction::new(&[funded_payer], message, blockhash)
}
