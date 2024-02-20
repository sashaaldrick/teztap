use serde_json::json;
use serde::Deserialize;
use reqwest::Error;

#[derive(Deserialize, Debug)]
pub struct ChallengeResponse {
    pub challenge: String,
    #[serde(rename = "challengeCounter")]
    pub challenge_counter: u32,
    #[serde(rename = "challengesNeeded")]
    pub challenges_needed: u32,
    pub difficulty: u32,
}

#[derive(Deserialize)]
pub struct VerifyResponse {
    pub challenge: String,
}

#[derive(Deserialize, Debug)]
pub struct TxHashResponse {
    #[serde(rename = "txHash")]
    pub tx_hash: String,
}

pub async fn challenge_request(client: &reqwest::Client, address: &str, amount: u32) -> Result<ChallengeResponse, Error> {
    let challenge_post_data = json!({
        "address": address, 
        "amount": amount
    });

    // Perform the POST request to get the challenge string
    let res = client
        .post("https://faucet.ghostnet.teztnets.com/challenge")
        .json(&challenge_post_data)
        .send()
        .await?;

    // Deserialize the response
    res.json().await

}

pub async fn verify_request(
    client: &reqwest::Client,
    address: &str,
    amount: u32,
    correct_hash: String,
    nonce: u32,
) -> Result<VerifyResponse, Error> {
    let verify_post_data = json!({
        "address": address, // Replace with the actual address you want to use
        "amount": amount,
        "nonce": nonce,
        "solution": correct_hash
    });

    // Perform the POST request to get the challenge string
    let res = client
        .post("https://faucet.ghostnet.teztnets.com/verify")
        .json(&verify_post_data)
        .send()
        .await?;

    // Deserialize the response into ChallengeResponse
    let verify_response = res.json().await;

    verify_response
}

pub async fn tx_hash_request(
    client: &reqwest::Client,
    address: &str,
    amount: u32,
    correct_hash: String,
    nonce: u32,
) -> Result<TxHashResponse, Error> {
    let verify_post_data = json!({
        "address": address,
        "amount": amount,
        "nonce": nonce,
        "solution": correct_hash
    });

    // Perform the POST request to get the challenge string
    let res = client
        .post("https://faucet.ghostnet.teztnets.com/verify")
        .json(&verify_post_data)
        .send()
        .await?;

    // Deserialize the response into ChallengeResponse
    let tx_hash_response: Result<TxHashResponse, Error> = res.json().await;

    tx_hash_response
}