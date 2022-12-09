pub mod client;

use solana_client::nonblocking::rpc_client::RpcClient;

use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::hash::Hash;
use solana_sdk::signature::Signature;
use solana_sdk::{
    message::Message, pubkey::Pubkey, signature::Keypair, signer::Signer, system_instruction,
    transaction::Transaction,
};

use self::client::LiteClient;

pub async fn new_funded_payer(lite_client: &LiteClient, amount: u64) -> Keypair {
    let payer = Keypair::new();

    // request airdrop to payer
    let airdrop_sig = lite_client
        .request_airdrop(&payer.pubkey(), amount)
        .await
        .unwrap();

    println!("{airdrop_sig}");

    while lite_client
        .get_signature_status_with_commitment(&airdrop_sig, CommitmentConfig::finalized())
        .await
        .unwrap()
        .is_none()
    {}

    payer
}

pub async fn wait_till_confirmed(lite_client: &LiteClient, sig: &Signature) {
    while lite_client.confirm_transaction(sig.to_string()).await {}
}

pub fn create_transaction(funded_payer: &Keypair, blockhash: Hash) -> Transaction {
    let to_pubkey = Pubkey::new_unique();

    // transfer instruction
    let instruction = system_instruction::transfer(&funded_payer.pubkey(), &to_pubkey, 1_000_000);

    let message = Message::new(&[instruction], Some(&funded_payer.pubkey()));

    Transaction::new(&[funded_payer], message, blockhash)
}

pub async fn generate_txs(
    num_of_txs: usize,
    rpc_client: &RpcClient,
    funded_payer: &Keypair,
) -> Vec<Transaction> {
    let mut txs = Vec::with_capacity(num_of_txs);

    let blockhash = rpc_client.get_latest_blockhash().await.unwrap();

    for _ in 0..num_of_txs {
        txs.push(create_transaction(funded_payer, blockhash));
    }

    txs
}
