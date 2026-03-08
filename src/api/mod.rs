pub mod crates_io;
pub mod github;
pub mod osv;

use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

pub async fn retry<T, E, F, Fut>(mut f: F, max_attempts: usize) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempts = 0;
    let mut backoff = Duration::from_secs(1);
    let max_backoff = Duration::from_secs(30);

    loop {
        attempts += 1;
        match f().await {
            Ok(val) => return Ok(val),
            Err(err) if attempts < max_attempts => {
                let jitter = rand::random::<f64>() * 0.5 + 0.75; // 0.75 to 1.25
                let sleep_duration = backoff.mul_f64(jitter);

                eprintln!(
                    "Attempt {} failed: {}. Retrying in {:?}...",
                    attempts, err, sleep_duration
                );

                sleep(sleep_duration).await;
                backoff = std::cmp::min(backoff * 2, max_backoff);
            }
            Err(err) => return Err(err),
        }
    }
}
