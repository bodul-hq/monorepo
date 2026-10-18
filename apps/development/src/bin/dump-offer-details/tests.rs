use super::*;
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

const OBJECT_ID: &str = "019a1234-5678-7000-8000-000000000001";
const EXTERNAL_ID: &str = "019a1234-5678-7000-8000-000000000002";
const BODY: &str = "<html>ž 🔎</html>";

fn fixture(path: &str, token: &str, status: u16, body: String) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let path = path.to_owned();
    let token = token.to_owned();
    let handle = thread::spawn(move || {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        let mut socket = loop {
            match listener.accept() {
                Ok((socket, _)) => break socket,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(std::time::Instant::now() < deadline, "request timed out");
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("{error}"),
            }
        };
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut input = Vec::new();
        while !input.windows(4).any(|w| w == b"\r\n\r\n") {
            let mut chunk = [0; 1024];
            let n = socket.read(&mut chunk).unwrap();
            assert!(n > 0);
            input.extend_from_slice(&chunk[..n]);
        }
        let request = String::from_utf8(input).unwrap();
        assert!(request.starts_with(&format!("GET {path} HTTP/1.1")));
        assert!(
            request
                .to_ascii_lowercase()
                .contains(&format!("authorization: bearer {token}\r\n"))
        );
        write!(socket, "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
    });
    (url, handle)
}

fn source() -> Source {
    Source {
        id: "source-id".into(),
        url: "https://www.anker.com/au/products/a1256".into(),
        retailer_code: "AnkerAu".into(),
        storage: Storage {
            content_size: BODY.len() as u64,
            object_storage_id: OBJECT_ID.parse().unwrap(),
            external_id: EXTERNAL_ID.parse().unwrap(),
        },
    }
}

fn record() -> Value {
    json!({"id": OBJECT_ID, "bucket": BUCKET, "idempotency_key": EXTERNAL_ID,
        "payload": BODY, "metadata": {}, "created_at": "2026-10-18T00:00:00Z"})
}

#[tokio::test]
async fn downloads_metadata_then_payload_and_skips_older_copies() {
    let (sourcing, sourcing_thread) = fixture(
        "/api/development/offer-details/sources?limit=200&offset=200",
        "sourcing-token",
        200,
        json!({"sources": [{"id":"source-id", "offer_id":"offer-id", "url":source().url,
            "retailer_code":"AnkerAu", "storage":{"content_size":BODY.len(),
                "object_storage_id":OBJECT_ID, "external_id":EXTERNAL_ID}}]})
        .to_string(),
    );
    let (storage, storage_thread) = fixture(
        &format!("/api/buckets/{BUCKET}/records/{OBJECT_ID}"),
        "storage-token",
        200,
        record().to_string(),
    );
    let pages = fetch_page(
        &reqwest::Client::new(),
        &format!("{sourcing}/api/development/offer-details/sources"),
        "sourcing-token",
        200,
    )
    .await
    .unwrap();
    sourcing_thread.join().unwrap();
    let objects = ObjectStorage::new(&storage, "storage-token", Transport::Rest).unwrap();
    let dir = env::temp_dir().join(format!("dump-offers-success-{}", process::id()));
    let mut written = HashMap::new();
    let Written::Newest(path) = write_source(&dir, &pages[0], &mut written, &objects)
        .await
        .unwrap()
    else {
        panic!("first page was not written")
    };
    storage_thread.join().unwrap();
    // The storage server has stopped: duplicate sources must not fetch again.
    assert!(matches!(
        write_source(&dir, &pages[0], &mut written, &objects)
            .await
            .unwrap(),
        Written::Older
    ));
    assert!(path.ends_with("ankerau/offer_details/sources/au-products-a1256.html"));
    assert_eq!(fs::read_to_string(path).unwrap(), BODY);
    fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn failed_or_mismatched_objects_do_not_write_files() {
    let mut responses = vec![
        (404, "{}".into()),
        (503, "{}".into()),
        (200, "malformed".into()),
    ];
    for (field, value) in [
        ("id", EXTERNAL_ID),
        ("bucket", "other"),
        ("idempotency_key", OBJECT_ID),
        ("payload", "wrong size"),
    ] {
        let mut object = record();
        object[field] = json!(value);
        responses.push((200, object.to_string()));
    }
    let dir = env::temp_dir().join(format!("dump-offers-failures-{}", process::id()));
    for (status, body) in responses {
        let (storage, handle) = fixture(
            &format!("/api/buckets/{BUCKET}/records/{OBJECT_ID}"),
            "storage-token",
            status,
            body,
        );
        let objects = ObjectStorage::new(&storage, "storage-token", Transport::Rest).unwrap();
        let mut written = HashMap::new();
        assert!(
            write_source(&dir, &source(), &mut written, &objects)
                .await
                .is_err()
        );
        handle.join().unwrap();
        assert!(written.is_empty());
        assert!(!dir.exists());
    }
}
