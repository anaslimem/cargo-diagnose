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
{
    let mut attempts = 0;
    let mut backoff = Duration::from_secs(1);

    loop {
        attempts += 1;
        match f().await {
            Ok(val) => return Ok(val),
            Err(_) if attempts < max_attempts => {
                sleep(backoff).await;
                backoff *= 2;
            }
            Err(err) => return Err(err),
        }
    }
}
