use sui_ns_lookup::SuinsClient;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const TEST_ADDRESS: &str = "0x0000000000000000000000000000000000000000000000000000000000000001";

#[tokio::test]
async fn reverse_lookup_returns_primary_name() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "data": ["alice.sui", "bob.sui"]
            }
        })))
        .mount(&server)
        .await;

    let client = SuinsClient::new(server.uri());
    let name = client.reverse_lookup(TEST_ADDRESS).await.unwrap();

    assert_eq!(name, Some("alice.sui".to_string()));
}

#[tokio::test]
async fn forward_lookup_returns_address() {
    let server = MockServer::start().await;
    let expected = TEST_ADDRESS;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": expected
        })))
        .mount(&server)
        .await;

    let client = SuinsClient::new(server.uri());
    let address = client.forward_lookup("alice.sui").await.unwrap();

    assert_eq!(address, Some(expected.to_string()));
}

#[tokio::test]
async fn reverse_lookup_rejects_invalid_address() {
    let server = MockServer::start().await;
    let client = SuinsClient::new(server.uri());

    let err = client.reverse_lookup("not-an-address").await.unwrap_err();
    assert!(err.to_string().contains("Invalid Sui address"));
}
