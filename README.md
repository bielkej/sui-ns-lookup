# sui-ns-lookup

Async SuiNS forward and reverse name resolution via Sui JSON-RPC.

## Install

From GitHub:

```toml
[dependencies]
sui_ns_lookup = { git = "https://github.com/bielkej/sui-ns-lookup" }
```

Or:

```bash
cargo add --git https://github.com/bielkej/sui-ns-lookup
```

## Usage

```rust
use sui_ns_lookup::SuinsClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = SuinsClient::new("https://fullnode.mainnet.sui.io:443");

    // Address -> name
    if let Some(name) = client.reverse_lookup("0x...").await? {
        println!("{name}");
    }

    // Name -> address
    if let Some(address) = client.forward_lookup("example.sui").await? {
        println!("{address}");
    }

    Ok(())
}
```

One-off lookups without constructing a client:

```rust
use sui_ns_lookup::{forward_suins_lookup, reverse_suins_lookup};

let name = reverse_suins_lookup("0x...", "https://fullnode.mainnet.sui.io:443").await?;
let address = forward_suins_lookup("example.sui", "https://fullnode.mainnet.sui.io:443").await?;
```

## RPC methods

| Direction | JSON-RPC method |
|-----------|-----------------|
| Address → name | `suix_resolveNameServiceNames` |
| Name → address | `suix_resolveNameServiceAddress` |

Requires a Sui fullnode RPC URL (public mainnet endpoint or your own node).

## Example

```bash
# Reverse lookup (address -> name)
cargo run --example lookup -- 0xYOUR_ADDRESS

# Forward lookup (name -> address)
cargo run --example lookup -- example.sui

# Custom RPC endpoint
cargo run --example lookup -- --rpc http://127.0.0.1:9000 example.sui
```

## License

MIT
