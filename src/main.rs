use hex::encode as hex_encode;
use reqwest::Error;
use serde::Deserialize;
use serde_json::json;
use openssl::hash::{hash, MessageDigest};
use std::time::Instant;
use tokio;

#[derive(Deserialize, Debug)]
struct ChallengeResponse {
    challenge: String,
    #[serde(rename = "challengeCounter")]
    challenge_counter: u32,
    #[serde(rename = "challengesNeeded")]
    challenges_needed: u32,
    difficulty: u32,
}

#[derive(Deserialize, Debug)]
struct VerifyResponse {
    status: String,
    challenge: String,
    #[serde(rename = "challengeCounter")]
    challenge_counter: u32,
    #[serde(rename = "challengesNeeded")]
    challenges_needed: u32,
    difficulty: u32,
    // status: String,
    // message: String
}

#[derive(Deserialize, Debug)]
struct TxHashResponse {
    #[serde(rename = "txHash")]
    tx_hash: String,
    status: String,
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let challenge_response = challenge_request().await?;
    let challenges_needed: u32 = challenge_response.challenges_needed;
    let mut counter = challenge_response.challenge_counter;
    let mut current_challenge_string = challenge_response.challenge;
    let difficulty = challenge_response.difficulty;

    println!("Difficulty: {}", difficulty);
    while counter <= challenges_needed {
        println!("Started solve challenge loop 🫡");
        println!("Current Counter: {}", counter);
        println!("Challenges Needed: {}", challenges_needed);

        let (correct_hash, nonce) = solve_challenge(&current_challenge_string, &difficulty);
        println!("Correct Hash/Nonce: {}/{}", correct_hash, nonce);

        if counter != challenges_needed {
            let verify_response = verify_request(correct_hash, nonce).await?;
            println!("Status of /verify request: {}", verify_response.status);
            counter += 1;
            current_challenge_string = verify_response.challenge;
        } else {
            // println!("Send manual request using: {}/{}", correct_hash, nonce);
            println!("In TxHashResponse Scope!");
            let tx_hash_response = tx_hash_request(correct_hash, nonce).await?;
            println!("Operation Hash: {}", tx_hash_response.tx_hash);
            counter += 1;
        }
    }

    Ok(())
}

async fn challenge_request() -> Result<ChallengeResponse, Error> {
    let client = reqwest::Client::new();

    let challenge_post_data = json!({
        "address": "tz1g8vkmcde6sWKaG2NN9WKzCkDM6Rziq194", // Replace with the actual address you want to use
        "amount": 1
    });

    // Perform the POST request to get the challenge string
    let res = client
        .post("https://faucet.ghostnet.teztnets.com/challenge")
        .json(&challenge_post_data)
        .send()
        .await?;

    // Deserialize the response into ChallengeResponse
    let challenge_response: Result<ChallengeResponse, Error> = res.json().await;

    challenge_response
}

async fn verify_request(correct_hash: String, nonce: u32) -> Result<VerifyResponse, Error> {
    let client = reqwest::Client::new();

    let verify_post_data = json!({
        "address": "tz1g8vkmcde6sWKaG2NN9WKzCkDM6Rziq194", // Replace with the actual address you want to use
        "amount": 1,
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
    let verify_response: Result<VerifyResponse, Error> = res.json().await;

    verify_response
}

async fn tx_hash_request(correct_hash: String, nonce: u32) -> Result<TxHashResponse, Error> {
    let client = reqwest::Client::new();

    let verify_post_data = json!({
        "address": "tz1g8vkmcde6sWKaG2NN9WKzCkDM6Rziq194", // Replace with the actual address you want to use
        "amount": 1,
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

// Computes the SHA-256 hash of the input string and returns a hexadecimal representation.
fn solve_challenge(challenge: &str, difficulty: &u32) -> (String, u32,) {
    let correct_hash;
    let mut nonce: u32 = 0;

    let start_time = Instant::now(); // Start the timer
    loop {
        let combined_string = format!("{}:{}", challenge, nonce.to_string());
        let result = hash(MessageDigest::sha256(), combined_string.as_bytes()).expect("Failed to compute hash");

        let zero_chars = result.iter().take_while(|&x| *x == 0).count() * 2;

        if zero_chars >= *difficulty as usize {
            correct_hash = hex_encode(result);
            break;
        }

        nonce += 1;
    }

    let duration = start_time.elapsed(); // Get the elapsed time
    println!("Time taken: {:.3} s", duration.as_secs_f64());

    (correct_hash, nonce)
}

