mod requests;
use requests::{challenge_request, ChallengeResponse, verify_request, tx_hash_request};

use hex::encode as hex_encode;
use openssl::hash::{hash, MessageDigest};
use reqwest::Error;
use std::time::Instant;
use tokio;
use indicatif::ProgressBar;

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
