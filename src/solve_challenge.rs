use hex::encode as hex_encode;
use openssl::hash::{hash, MessageDigest};

pub fn solve_challenge(challenge: &str, difficulty: &u32) -> (String, u32) {
    let correct_hash;
    let mut nonce: u32 = 0;
    let mut nonce_str = String::with_capacity(6);

    // TODO: multi-threaded
    // https://doc.rust-lang.org/std/thread/fn.available_parallelism.html
    // https://earthly.dev/blog/rust-concurrency-patterns-parallel-programming/
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
