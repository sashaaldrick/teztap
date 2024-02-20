mod requests;
use requests::{challenge_request, ChallengeResponse, verify_request, tx_hash_request};
mod solve_challenge;
use solve_challenge::solve_challenge;

use reqwest::Error;
use std::time::Instant;
use tokio;
use indicatif::ProgressBar;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = reqwest::Client::new();

    let address = String::from("tz1ZcrFLMV2LkyYpVvL49p5hmBRpoAHf8W4q");
    let amount: u32 = 10;

      let challenge_response = match challenge_request(&client, &address, amount).await {
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
            let verify_response = match verify_request(&client, &address, amount, correct_hash, nonce).await {
                Ok(response) => response,
                Err(e) => {
                    println!("The error is {}.", e);
                    return Err(e);
                }
            };

            current_challenge_string = verify_response.challenge;
            challenge_counter += 1;
        } else {
            let tx_hash_response = match tx_hash_request(&client, &address, amount, correct_hash, nonce).await {
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

