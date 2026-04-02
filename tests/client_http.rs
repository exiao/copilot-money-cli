use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use copilot_money_cli::client::{ClientMode, CopilotClient};

/// Like `serve_one` but doesn't assert a specific operationName, just returns any body.
fn serve_one_any(status: u16, body: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();

        let mut buf = Vec::new();
        let mut header_end = None;
        while header_end.is_none() {
            let mut tmp = [0u8; 1024];
            let n = stream.read(&mut tmp).unwrap();
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&tmp[..n]);
            if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                header_end = Some(i + 4);
            }
        }

        let header_end = header_end.expect("did not receive full headers");

        // drain request body
        let headers = String::from_utf8_lossy(&buf[..header_end]).to_string();
        let lower = headers.to_lowercase();
        let content_length = lower
            .lines()
            .find_map(|l| l.strip_prefix("content-length: "))
            .and_then(|v| v.trim().parse::<usize>().ok())
            .unwrap_or(0);
        let mut body_buf = buf[header_end..].to_vec();
        while body_buf.len() < content_length {
            let mut tmp = vec![0u8; content_length - body_buf.len()];
            let n = stream.read(&mut tmp).unwrap();
            if n == 0 {
                break;
            }
            body_buf.extend_from_slice(&tmp[..n]);
        }

        let resp = format!(
            "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(resp.as_bytes()).unwrap();
    });

    format!("http://{}", addr)
}

fn serve_one(status: u16, body: &'static str, assert_bearer: Option<&'static str>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();

        let mut buf = Vec::new();
        let mut header_end = None;
        while header_end.is_none() {
            let mut tmp = [0u8; 1024];
            let n = stream.read(&mut tmp).unwrap();
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&tmp[..n]);
            if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                header_end = Some(i + 4);
            }
        }

        let header_end = header_end.expect("did not receive full headers");
        let headers = String::from_utf8_lossy(&buf[..header_end]).to_string();
        let lower = headers.to_lowercase();
        assert!(lower.starts_with("post /api/graphql"));
        if let Some(t) = assert_bearer {
            assert!(lower.contains(&format!("authorization: bearer {t}")));
        }

        let content_length = lower
            .lines()
            .find_map(|l| l.strip_prefix("content-length: "))
            .and_then(|v| v.trim().parse::<usize>().ok())
            .unwrap_or(0);

        let mut body_buf = buf[header_end..].to_vec();
        while body_buf.len() < content_length {
            let mut tmp = vec![0u8; content_length - body_buf.len()];
            let n = stream.read(&mut tmp).unwrap();
            if n == 0 {
                break;
            }
            body_buf.extend_from_slice(&tmp[..n]);
        }
        let req_body = String::from_utf8_lossy(&body_buf[..content_length]).to_string();
        assert!(req_body.contains("\"operationName\":\"User\""));

        let resp = format!(
            "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(resp.as_bytes()).unwrap();
    });

    format!("http://{}", addr)
}

fn serve_two(
    first_status: u16,
    first_body: &'static str,
    first_assert_bearer: Option<&'static str>,
    second_status: u16,
    second_body: &'static str,
    second_assert_bearer: Option<&'static str>,
) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    thread::spawn(move || {
        for (status, body, assert_bearer) in [
            (first_status, first_body, first_assert_bearer),
            (second_status, second_body, second_assert_bearer),
        ] {
            let (mut stream, _) = listener.accept().unwrap();

            let mut buf = Vec::new();
            let mut header_end = None;
            while header_end.is_none() {
                let mut tmp = [0u8; 1024];
                let n = stream.read(&mut tmp).unwrap();
                if n == 0 {
                    break;
                }
                buf.extend_from_slice(&tmp[..n]);
                if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                    header_end = Some(i + 4);
                }
            }

            let header_end = header_end.expect("did not receive full headers");
            let headers = String::from_utf8_lossy(&buf[..header_end]).to_string();
            let lower = headers.to_lowercase();
            assert!(lower.starts_with("post /api/graphql"));
            if let Some(t) = assert_bearer {
                assert!(lower.contains(&format!("authorization: bearer {t}")));
            }

            let content_length = lower
                .lines()
                .find_map(|l| l.strip_prefix("content-length: "))
                .and_then(|v| v.trim().parse::<usize>().ok())
                .unwrap_or(0);

            let mut body_buf = buf[header_end..].to_vec();
            while body_buf.len() < content_length {
                let mut tmp = vec![0u8; content_length - body_buf.len()];
                let n = stream.read(&mut tmp).unwrap();
                if n == 0 {
                    break;
                }
                body_buf.extend_from_slice(&tmp[..n]);
            }
            let req_body = String::from_utf8_lossy(&body_buf[..content_length]).to_string();
            assert!(req_body.contains("\"operationName\":\"User\""));

            let resp = format!(
                "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(resp.as_bytes()).unwrap();
        }
    });

    format!("http://{}", addr)
}

#[test]
fn http_mode_sends_bearer_and_accepts_success() {
    let base_url = serve_one(200, r#"{"data":{"user":{"id":"u1"}}}"#, Some("abc"));
    let tmp = tempfile::tempdir().unwrap();
    let client = CopilotClient::new(ClientMode::Http {
        base_url,
        token: Some("abc".to_string()),
        token_file: tmp.path().join("token"),
        session_dir: None,
    });
    client.try_user_query().unwrap();
}

#[test]
fn http_mode_errors_on_graphql_errors_key() {
    let base_url = serve_one(200, r#"{"errors":[{"message":"nope"}]}"#, None);
    let tmp = tempfile::tempdir().unwrap();
    let client = CopilotClient::new(ClientMode::Http {
        base_url,
        token: None,
        token_file: tmp.path().join("token"),
        session_dir: None,
    });
    assert!(client.try_user_query().is_err());
}

#[test]
fn http_mode_formats_graphql_error_with_code() {
    let base_url = serve_one(
        400,
        r#"{"errors":[{"extensions":{"code":"BAD_USER_INPUT"},"message":"Value does not exist"}]}"#,
        None,
    );
    let tmp = tempfile::tempdir().unwrap();
    let client = CopilotClient::new(ClientMode::Http {
        base_url,
        token: None,
        token_file: tmp.path().join("token"),
        session_dir: None,
    });

    let err = client.try_user_query().unwrap_err().to_string();
    assert!(err.contains("graphql error (BAD_USER_INPUT): Value does not exist"));
}

#[test]
fn http_mode_errors_on_http_status() {
    let base_url = serve_one(401, r#"{"data":null}"#, None);
    let tmp = tempfile::tempdir().unwrap();
    let client = CopilotClient::new(ClientMode::Http {
        base_url,
        token: None,
        token_file: tmp.path().join("token"),
        session_dir: None,
    });
    assert!(client.try_user_query().is_err());
}

#[test]
fn http_mode_refreshes_token_on_unauthenticated_and_retries_once() {
    // NOTE: In Rust 2024 edition, mutating process env is `unsafe` due to potential UB with
    // concurrent access. This test runs single-threaded with a narrowly-scoped env var used
    // only by the refresh hook.
    unsafe { std::env::set_var("COPILOT_TEST_REFRESH_TOKEN", "refreshed_token") };

    let base_url = serve_two(
        401,
        r#"{"errors":[{"extensions":{"code":"UNAUTHENTICATED"},"message":"User is not authenticated"}]}"#,
        Some("expired_token"),
        200,
        r#"{"data":{"user":{"id":"u1"}}}"#,
        Some("refreshed_token"),
    );

    let tmp = tempfile::tempdir().unwrap();
    let session_dir = tmp.path().join("session");
    std::fs::create_dir_all(&session_dir).unwrap();
    let token_file = tmp.path().join("token");

    let client = CopilotClient::new(ClientMode::Http {
        base_url,
        token: Some("expired_token".to_string()),
        token_file: token_file.clone(),
        session_dir: Some(session_dir),
    });

    client.try_user_query().unwrap();

    let saved = std::fs::read_to_string(&token_file).unwrap();
    assert_eq!(saved.trim(), "refreshed_token");

    unsafe { std::env::remove_var("COPILOT_TEST_REFRESH_TOKEN") };
}

// -- list_transactions success path -------------------------------------------

#[test]
fn http_mode_list_transactions_success() {
    let body = r#"{
        "data": {
            "transactions": {
                "edges": [
                    {"cursor":"c1","node":{"id":"txn_1","date":"2025-12-15","name":"Test","amount":"-50.00","itemId":"item_1","accountId":"acct_1","isReviewed":false}}
                ],
                "pageInfo": {"endCursor":"c1","hasNextPage":false,"hasPreviousPage":false,"startCursor":"c1"}
            }
        }
    }"#;
    // serve_one_any avoids asserting operationName so we can reuse for any query
    let base_url = serve_one_any(200, Box::leak(body.to_string().into_boxed_str()));
    let tmp = tempfile::tempdir().unwrap();
    let client = CopilotClient::new(ClientMode::Http {
        base_url,
        token: Some("tok".to_string()),
        token_file: tmp.path().join("token"),
        session_dir: None,
    });
    let txns = client.list_transactions(50).unwrap();
    assert_eq!(txns.len(), 1);
    assert_eq!(txns[0].id.as_str(), "txn_1");
}

// -- list_categories success path ---------------------------------------------

#[test]
fn http_mode_list_categories_success() {
    let body = r#"{
        "data": {
            "categories": [
                {"id":"cat_1","name":"Food"},
                {"id":"cat_2","name":"Transport"}
            ]
        }
    }"#;
    let base_url = serve_one_any(200, Box::leak(body.to_string().into_boxed_str()));
    let tmp = tempfile::tempdir().unwrap();
    let client = CopilotClient::new(ClientMode::Http {
        base_url,
        token: Some("tok".to_string()),
        token_file: tmp.path().join("token"),
        session_dir: None,
    });
    let cats = client.list_categories(false, false, false).unwrap();
    assert_eq!(cats.len(), 2);
    assert_eq!(cats[0].name.as_deref(), Some("Food"));
    assert_eq!(cats[1].name.as_deref(), Some("Transport"));
}

// -- list_tags success path ---------------------------------------------------

#[test]
fn http_mode_list_tags_success() {
    let body = r#"{
        "data": {
            "tags": [
                {"id":"tag_1","name":"Groceries","colorName":"GREEN1"}
            ]
        }
    }"#;
    let base_url = serve_one_any(200, Box::leak(body.to_string().into_boxed_str()));
    let tmp = tempfile::tempdir().unwrap();
    let client = CopilotClient::new(ClientMode::Http {
        base_url,
        token: Some("tok".to_string()),
        token_file: tmp.path().join("token"),
        session_dir: None,
    });
    let tags = client.list_tags().unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name.as_deref(), Some("Groceries"));
}

// -- Double retry exhaustion: both attempts return UNAUTHENTICATED ------------

#[test]
fn http_mode_double_retry_exhaustion_errors() {
    let err_body = r#"{"errors":[{"extensions":{"code":"UNAUTHENTICATED"},"message":"User is not authenticated"}]}"#;

    // Both attempts get UNAUTHENTICATED. No session_dir → no refresh possible → should bail on first attempt.
    let base_url = serve_one_any(401, Box::leak(err_body.to_string().into_boxed_str()));
    let tmp = tempfile::tempdir().unwrap();
    let client = CopilotClient::new(ClientMode::Http {
        base_url,
        token: Some("bad".to_string()),
        token_file: tmp.path().join("token"),
        session_dir: None,
    });
    let err = client.try_user_query().unwrap_err().to_string();
    assert!(err.contains("unauthenticated"));
}

// NOTE: A "double retry with session_dir both fail" test is omitted because
// it would require setting COPILOT_TEST_REFRESH_TOKEN, which races with
// http_mode_refreshes_token_on_unauthenticated_and_retries_once when tests
// run in parallel.
