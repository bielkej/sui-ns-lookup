use std::env;

use sui_ns_lookup::SuinsClient;

const DEFAULT_RPC: &str = "https://fullnode.mainnet.sui.io:443";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        eprintln!("Usage: lookup [--rpc URL] <address-or-name>");
        eprintln!();
        eprintln!("  Reverse lookup: pass a 0x address");
        eprintln!("  Forward lookup: pass a .sui name");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  cargo run --example lookup -- 0xabc...def");
        eprintln!("  cargo run --example lookup -- example.sui");
        eprintln!("  cargo run --example lookup -- --rpc http://127.0.0.1:9000 example.sui");
        return Ok(());
    }

    let (rpc_url, query) = parse_args(&args)?;
    let client = SuinsClient::new(rpc_url);

    if query.starts_with("0x") {
        match client.reverse_lookup(&query).await? {
            Some(name) => println!("{name}"),
            None => println!("No SuiNS name found for {query}"),
        }
    } else {
        match client.forward_lookup(&query).await? {
            Some(address) => println!("{address}"),
            None => println!("No address found for {query}"),
        }
    }

    Ok(())
}

fn parse_args(args: &[String]) -> anyhow::Result<(&str, String)> {
    if args[0] == "--rpc" {
        let rpc_url = args
            .get(1)
            .ok_or_else(|| anyhow::anyhow!("--rpc requires a URL argument"))?;
        let query = args
            .get(2)
            .ok_or_else(|| anyhow::anyhow!("missing address or name after --rpc URL"))?
            .clone();
        Ok((rpc_url, query))
    } else {
        Ok((DEFAULT_RPC, args[0].clone()))
    }
}
