pub mod client;
pub mod metrics;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::bail;
use log::info;
use solana_client::nonblocking::rpc_client::RpcClient;

use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::hash::Hash;
use solana_sdk::signature::Signature;
use solana_sdk::{
    message::Message, pubkey::Pubkey, signature::Keypair, signer::Signer, system_instruction,
    transaction::Transaction,
};
use tokio::time::Instant;

use self::client::LiteClient;

pub async fn new_funded_payer(lite_client: &LiteClient, amount: u64) -> anyhow::Result<Keypair> {
    let payer = Keypair::new();
    let payer_pubkey = payer.pubkey().to_string();

    // request airdrop to payer
    let airdrop_sig = lite_client.request_airdrop(&payer.pubkey(), amount).await?;

    info!("Air Dropping {payer_pubkey} with {amount}L");

   thread::sleep(Duration::from_secs(20));

    //loop {
    //    if let Some(res) = lite_client
    //        .get_signature_status_with_commitment(&airdrop_sig, CommitmentConfig::finalized())
    //        .await?
    //    {
    //        match res {
    //            Ok(_) => break,
    //            Err(_) => bail!("Error air dropping {payer_pubkey}"),
    //        }
    //    }
    //}

    info!("Air Drop Successful: {airdrop_sig}");

    Ok(payer)
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
) -> anyhow::Result<Vec<Transaction>> {
    let mut txs = Vec::with_capacity(num_of_txs);

    let blockhash = rpc_client.get_latest_blockhash().await?;

    for _ in 0..num_of_txs {
        txs.push(create_transaction(funded_payer, blockhash));
    }

    Ok(txs)
}

pub struct LatestBlockHash {
    lite_client: Arc<LiteClient>,
    blockhash: Hash,
    ttl: Duration,
    last_fetch_stamp: Instant,
}

impl LatestBlockHash {
    pub async fn new(lite_client: Arc<LiteClient>, ttl: Duration) -> anyhow::Result<Self> {
        Ok(Self {
            blockhash: lite_client.get_latest_blockhash().await?,
            lite_client,
            ttl,
            last_fetch_stamp: Instant::now(),
        })
    }

    pub async fn get_latest_blockhash(&mut self) -> anyhow::Result<&Hash> {
        if self.last_fetch_stamp.elapsed() > self.ttl {
            let blockhash = self.lite_client.get_latest_blockhash().await?;
            self.blockhash = blockhash;
            self.last_fetch_stamp = Instant::now();
        }

        Ok(&self.blockhash)
    }
}
