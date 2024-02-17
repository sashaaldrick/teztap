use hex::encode as hex_encode;
use reqwest::Error;
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
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

// #[derive(Deserialize, Debug)]
// struct VerifyResponse {
//     tx_hash: String,
// }

#[tokio::main]
async fn main() {
    let challenge_response = challenge_request().await;

    println!("Challenges Needed: {:?}", &challenge_response);

    match challenge_response {
        Err(error) => println!("{}", error),
        Ok(response) => {
            let (correct_hash, nonce) = solve_challenge(&response.challenge, &response.difficulty);
            println!("Correct Hash/Nonce: {}/{}", correct_hash, nonce);

            match verify_request(correct_hash, nonce).await {
                Err(error) => println!("{}", error),
                Ok(response) => {
                    println!(
                        "Challenge/Challenge Counter: {}/{}",
                        &response.challenge, &response.challenge_counter
                    )
                }
            }
        }
    }

    // // Compute the SHA-256 hash of the challenge string
    // println!(
    //     "SHA-256 Hash of Combined Challenge String/Nonce: {}/{}",
    //     correct_hash,
    //     nonce
    // );

    // let verify_post_data = json!({
    //     "address": "tz1MwTpHXayNxvv8R3WrmQSvCqqfJZLyn3Yt", // Replace with the actual address you want to use
    //     "nonce": nonce,
    //     "solution": correct_hash
    // });

    // // Perform the POST request to verify and recieve tez
    // let res = client
    //     .post("https://faucet.ghostnet.teztnets.com/verify")
    //     .json(&verify_post_data)
    //     .send()
    //     .await?;

    // let verify_response: ChallengeResponse = res.json().await?;

    // Ok(())
}

async fn challenge_request() -> Result<ChallengeResponse, Error> {
    let client = reqwest::Client::new();

    let challenge_post_data = json!({
        "address": "tz1ZcrFLMV2LkyYpVvL49p5hmBRpoAHf8W4q", // Replace with the actual address you want to use
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
        "address": "tz1ZcrFLMV2LkyYpVvL49p5hmBRpoAHf8W4q", // Replace with the actual address you want to use
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

// Computes the SHA-256 hash of the input string and returns a hexadecimal representation.
fn solve_challenge(challenge: &str, difficulty: &u32) -> (String, u32) {
    let correct_hash;
    let mut nonce: u32 = 0;

    let start_time = Instant::now(); // Start the timer

    loop {
        let combined_string = format!("{}:{}", challenge, nonce.to_string());

        let mut hasher = Sha256::new();
        hasher.update(combined_string);
        let result = hasher.finalize(); // This is the hash result as bytes

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

// GET REQUEST
// async fn main() -> Result<(), Error> {
//     let response = reqwest::get("https://faucet.ghostnet.teztnets.com/info")
//         .await?
//         .text()
//         .await?;

//     println!("Response: {}", response);
//     Ok(())
// }
