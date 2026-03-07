use cargo_diagnose::api::github::parse_github_url;
use cargo_diagnose::api::retry;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_retry_success_first_try() {
    let mut count = 0;
    let res: Result<i32, &str> = retry(
        || {
            count += 1;
            async move { Ok(42) }
        },
        3,
    )
    .await;

    assert_eq!(res, Ok(42));
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_retry_eventual_success() {
    let count = Arc::new(Mutex::new(0));
    let count_clone = Arc::clone(&count);

    let res: Result<i32, &str> = retry(
        || {
            let count_inner = Arc::clone(&count_clone);
            async move {
                let mut c = count_inner.lock().await;
                *c += 1;
                if *c < 3 { Err("fail") } else { Ok(42) }
            }
        },
        3,
    )
    .await;

    assert_eq!(res, Ok(42));
    assert_eq!(*count.lock().await, 3);
}

#[tokio::test]
async fn test_retry_max_attempts_reached() {
    let count = Arc::new(Mutex::new(0));
    let count_clone = Arc::clone(&count);

    let res: Result<i32, &str> = retry(
        || {
            let count_inner = Arc::clone(&count_clone);
            async move {
                let mut c = count_inner.lock().await;
                *c += 1;
                Err("always fail")
            }
        },
        2,
    )
    .await;

    assert_eq!(res, Err("always fail"));
    assert_eq!(*count.lock().await, 2);
}

#[test]
fn test_parse_github_url() {
    assert_eq!(
        parse_github_url("https://github.com/tokio-rs/tokio"),
        Some(("tokio-rs".to_string(), "tokio".to_string()))
    );
    assert_eq!(
        parse_github_url("https://github.com/tokio-rs/tokio.git"),
        Some(("tokio-rs".to_string(), "tokio".to_string()))
    );
    assert_eq!(
        parse_github_url("http://github.com/serde-rs/serde"),
        Some(("serde-rs".to_string(), "serde".to_string()))
    );
    assert_eq!(parse_github_url("github.com/tokio-rs/tokio"), None);
    assert_eq!(parse_github_url("https://gitlab.com/example/repo"), None);
    assert_eq!(parse_github_url("https://github.com/only-owner"), None);
}
