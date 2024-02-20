use hex::encode as hex_encode;
use openssl::hash::{hash, MessageDigest};
use reqwest::Error;
use serde::Deserialize;
use serde_json::json;
use std::time::Instant;
use tokio;
use indicatif::ProgressBar;

#[derive(Deserialize, Debug)]
struct ChallengeResponse {
    challenge: String,
    #[serde(rename = "challengeCounter")]
    challenge_counter: u32,
    #[serde(rename = "challengesNeeded")]
    challenges_needed: u32,
    difficulty: u32,
}

#[derive(Deserialize)]
struct VerifyResponse {
    challenge: String,
}

#[derive(Deserialize, Debug)]
struct TxHashResponse {
    #[serde(rename = "txHash")]
    tx_hash: String,
    // status: String,
    // message: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = reqwest::Client::new();
  
    let challenge_response = match challenge_request(&client).await {
        Ok(response) => response,
        Err(e) => {
            println!("The error is {}.", e);
            return Err(e);
        }
    };

    let ChallengeResponse {challenge, mut challenge_counter, challenges_needed, difficulty} = challenge_response;
    let mut current_challenge_string = challenge.clone();

    // debug: print, progress bar, timing
    println!("Challenges Needed: {}", challenges_needed);
    let pb = ProgressBar::new(challenges_needed as u64);
    let start_time = Instant::now(); // Start the timer

    // cycle solving challenges
    while challenge_counter <= challenges_needed {
        let (correct_hash, nonce) = solve_challenge(&current_challenge_string, &difficulty);

        if challenge_counter != challenges_needed {
            let verify_response = match verify_request(&client, correct_hash, nonce).await {
                Ok(response) => response,
                Err(e) => {
                    println!("The error is {}.", e);
                    return Err(e);
                }
            };

            current_challenge_string = verify_response.challenge;
            challenge_counter += 1;
        } else {
            let tx_hash_response = match tx_hash_request(&client, correct_hash, nonce).await {
                Ok(response) => response,
                Err(e) => {
                    println!("The error is {}.", e);
                    return Err(e);
                }
            };
            match open::that(format!("https://ghostnet.tzkt.io/{}/", tx_hash_response.tx_hash)) {
                Ok(_) => (),
                Err(e) => {
                    println!("Failed to open URL: {}", e);
                }
            };
            challenge_counter += 1;
        }

        pb.inc(1);
    }

    let duration = start_time.elapsed(); // Get the elapsed time

    pb.finish_with_message("Completed!");
    println!("Cumulative time taken: {:.3} s", duration.as_secs_f64());

   Ok(())
}

async fn challenge_request(client: &reqwest::Client) -> Result<ChallengeResponse, Error> {
    let challenge_post_data = json!({
        "address": "tz1ZcrFLMV2LkyYpVvL49p5hmBRpoAHf8W4q", // Replace with the actual address you want to use
        "amount": 10
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

async fn verify_request(
    client: &reqwest::Client,
    correct_hash: String,
    nonce: u32,
) -> Result<VerifyResponse, Error> {
    let verify_post_data = json!({
        "address": "tz1ZcrFLMV2LkyYpVvL49p5hmBRpoAHf8W4q", // Replace with the actual address you want to use
        "amount": 10,
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

async fn tx_hash_request(
    client: &reqwest::Client,
    correct_hash: String,
    nonce: u32,
) -> Result<TxHashResponse, Error> {
    let verify_post_data = json!({
        "address": "tz1ZcrFLMV2LkyYpVvL49p5hmBRpoAHf8W4q", // Replace with the actual address you want to use
        "amount": 10,
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
fn solve_challenge(challenge: &str, difficulty: &u32) -> (String, u32) {
    let correct_hash;
    let mut nonce: u32 = 0;
    let mut nonce_str = String::with_capacity(6);

    // let start_time = Instant::now(); // Start the timer
    loop {
        nonce_str.clear(); // Clear the string
        nonce_str.push_str(&nonce.to_string());

        let combined_string = format!("{}:{}", challenge, nonce.to_string());
        let result = hash(MessageDigest::sha256(), combined_string.as_bytes())
            .expect("Failed to compute hash");

        let zero_chars = result.iter().take_while(|&x| *x == 0).count() * 2;

        if zero_chars >= *difficulty as usize {
            correct_hash = hex_encode(result);
            break;
        }

        nonce += 1;
    }

    // let duration = start_time.elapsed(); // Get the elapsed time
    // println!("Time taken: {:.3} s", duration.as_secs_f64());

    (correct_hash, nonce)
}
