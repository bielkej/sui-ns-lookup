use anyhow::{bail, Result};
use serde_json::{json, Value};

/// Client for SuiNS forward and reverse lookups via a Sui JSON-RPC endpoint.
pub struct SuinsClient {
    rpc_url: String,
    http: reqwest::Client,
}

impl SuinsClient {
    /// Create a client targeting the given Sui fullnode RPC URL.
    pub fn new(rpc_url: impl Into<String>) -> Self {
        Self::with_client(rpc_url, reqwest::Client::new())
    }

    /// Create a client with a custom HTTP client (useful for timeouts or testing).
    pub fn with_client(rpc_url: impl Into<String>, http: reqwest::Client) -> Self {
        Self {
            rpc_url: rpc_url.into(),
            http,
        }
    }

    /// Reverse lookup: resolve a Sui address to its primary SuiNS name.
    ///
    /// Uses the `suix_resolveNameServiceNames` JSON-RPC method.
    /// Returns `None` when the address has no registered name.
    pub async fn reverse_lookup(&self, address: &str) -> Result<Option<String>> {
        validate_sui_address(address)?;

        let result = json_rpc(
            &self.http,
            &self.rpc_url,
            "suix_resolveNameServiceNames",
            json!([address]),
        )
        .await?;

        parse_reverse_result(&result)
    }

    /// Forward lookup: resolve a SuiNS name to its Sui address.
    ///
    /// Uses the `suix_resolveNameServiceAddress` JSON-RPC method.
    /// Returns `None` when the name is not registered.
    pub async fn forward_lookup(&self, name: &str) -> Result<Option<String>> {
        let result = json_rpc(
            &self.http,
            &self.rpc_url,
            "suix_resolveNameServiceAddress",
            json!([name]),
        )
        .await?;

        parse_forward_result(&result)
    }
}

/// Reverse lookup using a one-off client for the given RPC URL.
pub async fn reverse_suins_lookup(address: &str, rpc_url: &str) -> Result<Option<String>> {
    SuinsClient::new(rpc_url).reverse_lookup(address).await
}

/// Forward lookup using a one-off client for the given RPC URL.
pub async fn forward_suins_lookup(name: &str, rpc_url: &str) -> Result<Option<String>> {
    SuinsClient::new(rpc_url).forward_lookup(name).await
}

fn validate_sui_address(address: &str) -> Result<()> {
    let hex = address
        .strip_prefix("0x")
        .ok_or_else(|| anyhow::anyhow!("Invalid Sui address '{}': must start with 0x", address))?;

    if hex.len() != 64 {
        bail!(
            "Invalid Sui address '{}': expected 64 hex characters after 0x, got {}",
            address,
            hex.len()
        );
    }

    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!(
            "Invalid Sui address '{}': contains non-hex characters",
            address
        );
    }

    Ok(())
}

async fn json_rpc(
    http: &reqwest::Client,
    rpc_url: &str,
    method: &str,
    params: Value,
) -> Result<Value> {
    let request_body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    });

    let response = http
        .post(rpc_url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        bail!("RPC request failed with status: {}", response.status());
    }

    let response_body: Value = response.json().await?;

    if let Some(error) = response_body.get("error") {
        let message = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");
        bail!("RPC error: {}", message);
    }

    response_body
        .get("result")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Invalid response format: missing result field"))
}

fn parse_reverse_result(result: &Value) -> Result<Option<String>> {
    let Some(data) = result.get("data") else {
        return Ok(None);
    };

    let Some(names_array) = data.as_array() else {
        return Ok(None);
    };

    let Some(first_name) = names_array.first().and_then(|n| n.as_str()) else {
        return Ok(None);
    };

    Ok(Some(first_name.to_string()))
}

fn parse_forward_result(result: &Value) -> Result<Option<String>> {
    Ok(result.as_str().map(str::to_string))
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn validate_accepts_valid_address() {
        let addr = "0x".to_string() + &"a".repeat(64);
        assert!(validate_sui_address(&addr).is_ok());
    }

    #[test]
    fn validate_rejects_missing_prefix() {
        assert!(validate_sui_address(&"a".repeat(64)).is_err());
    }

    #[test]
    fn validate_rejects_wrong_length() {
        assert!(validate_sui_address("0xabcd").is_err());
    }

    #[test]
    fn parse_reverse_extracts_first_name() {
        let result = json!({ "data": ["alice.sui", "bob.sui"] });
        assert_eq!(
            parse_reverse_result(&result).unwrap(),
            Some("alice.sui".to_string())
        );
    }

    #[test]
    fn parse_reverse_returns_none_for_empty() {
        let result = json!({ "data": [] });
        assert_eq!(parse_reverse_result(&result).unwrap(), None);
    }

    #[test]
    fn parse_forward_extracts_address() {
        let addr = "0x".to_string() + &"1".repeat(64);
        let result = json!(addr);
        assert_eq!(parse_forward_result(&result).unwrap(), Some(addr));
    }
}
