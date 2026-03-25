//! CLI for tx_tracker
//! 
//! Build with CLI: cargo build --features cli
//! Build without CLI (DLL only): cargo build --no-default-features

#[cfg(feature = "cli")]
mod cli {
    use clap::Parser;
    use tx_tracker::{Request, Tracker};
    use serde_json::json;

    #[derive(Parser)]
    #[command(author, version, about, long_about = None)]
    pub struct Cli {
        /// Domain: transaction / activity
        #[arg(long)]
        pub domain: String,

        /// Tool/action: create / list / update / delete
        #[arg(long)]
        pub tool: String,

        // Transaction fields
        #[arg(long)]
        pub amount: Option<f64>,
        #[arg(long)]
        pub kind: Option<String>,
        #[arg(long)]
        pub desc: Option<String>,
        #[arg(long)]
        pub id: Option<i64>,

        // Activity fields
        #[arg(long)]
        pub start: Option<String>,
        #[arg(long)]
        pub stop: Option<String>,
        #[arg(long)]
        pub activity_desc: Option<String>,

        // Shared category field
        #[arg(long)]
        pub category: Option<String>,

        // Shared place field
        #[arg(long)]
        pub place: Option<String>,
    }

    pub fn run() -> anyhow::Result<()> {
        #[tokio::main]
        async fn inner() -> anyhow::Result<()> {
            let cli = Cli::parse();
            let tracker = Tracker::new(None).await?;

            let request = build_request(&cli)?;
            let response = tracker.handle(&request).await;

            if response.success {
                match cli.domain.as_str() {
                    "transaction" => print_transaction_response(&response, &cli.tool),
                    "activity" => print_activity_response(&response, &cli.tool),
                    _ => {}
                }
            } else {
                eprintln!("Error: {}", response.error.unwrap_or_default());
                std::process::exit(1);
            }

            Ok(())
        }

        inner()
    }

    fn build_request(cli: &Cli) -> anyhow::Result<Request> {
        match cli.domain.as_str() {
            "transaction" => match cli.tool.as_str() {
                "create" => {
                    let amount = cli.amount.ok_or_else(|| anyhow::anyhow!("--amount is required"))?;
                    let kind = cli.kind.clone().ok_or_else(|| anyhow::anyhow!("--kind is required"))?;
                    let desc = cli.desc.clone().unwrap_or_default();
                    let mut args = json!({
                        "amount": amount,
                        "kind": kind,
                        "description": desc
                    });
                    if let Some(cat) = &cli.category {
                        args["category"] = json!(cat);
                    }
                    if let Some(place) = &cli.place {
                        args["place"] = json!(place);
                    }
                    Ok(Request {
                        tool: "create_transaction".into(),
                        args,
                    })
                }
                "list" => {
                    let mut args = json!({});
                    if let Some(kind) = &cli.kind {
                        args["kind"] = json!(kind);
                    }
                    Ok(Request {
                        tool: "list_transactions".into(),
                        args,
                    })
                }
                "update" => {
                    let id = cli.id.ok_or_else(|| anyhow::anyhow!("--id is required"))?;
                    let mut args = json!({
                        "id": id,
                    });
                    if let Some(amount) = cli.amount {
                        args["amount"] = json!(amount);
                    }
                    if let Some(kind) = &cli.kind {
                        args["kind"] = json!(kind);
                    }
                    if let Some(desc) = &cli.desc {
                        args["description"] = json!(desc);
                    }
                    if let Some(cat) = &cli.category {
                        args["category"] = json!(cat);
                    }
                    if let Some(place) = &cli.place {
                        args["place"] = json!(place);
                    }
                    Ok(Request {
                        tool: "update_transaction".into(),
                        args,
                    })
                }
                "delete" => {
                    let id = cli.id.ok_or_else(|| anyhow::anyhow!("--id is required"))?;
                    Ok(Request {
                        tool: "delete_transaction".into(),
                        args: json!({ "id": id }),
                    })
                }
                _ => Err(anyhow::anyhow!("Unknown tool: {}", cli.tool)),
            },
            "activity" => match cli.tool.as_str() {
                "create" => {
                    let start = cli.start.clone().ok_or_else(|| anyhow::anyhow!("--start is required"))?;
                    let stop = cli.stop.clone().ok_or_else(|| anyhow::anyhow!("--stop is required"))?;
                    let desc = cli.activity_desc.clone().unwrap_or_default();
                    let mut args = json!({
                        "start_time": start,
                        "stop_time": stop,
                        "description": desc
                    });
                    if let Some(cat) = &cli.category {
                        args["category"] = json!(cat);
                    }
                    if let Some(place) = &cli.place {
                        args["place"] = json!(place);
                    }
                    Ok(Request {
                        tool: "create_activity".into(),
                        args,
                    })
                }
                "list" => Ok(Request {
                    tool: "list_activities".into(),
                    args: json!(null),
                }),
                "update" | "delete" => Err(anyhow::anyhow!("Activity update/delete not yet implemented")),
                _ => Err(anyhow::anyhow!("Unknown tool: {}", cli.tool)),
            },
            _ => Err(anyhow::anyhow!("Unknown domain: {}", cli.domain)),
        }
    }

    fn print_transaction_response(response: &tx_tracker::Response, tool: &str) {
        match tool {
            "create" => println!("Transaction added!"),
            "list" => {
                if let Some(data) = &response.data {
                    if let Some(txs) = data.as_array() {
                        for tx in txs {
                            let id = tx["id"].as_i64().unwrap_or(0);
                            let kind = tx["kind"].as_str().unwrap_or("");
                            let desc = tx["description"].as_str().unwrap_or("");
                            let amount = tx["amount"].as_f64().unwrap_or(0.0);
                            println!("[{}] {} - {} ({})", id, kind, desc, amount);
                        }
                    }
                }
            }
            "update" => println!("Transaction updated!"),
            "delete" => println!("Transaction deleted!"),
            _ => {}
        }
    }

    fn print_activity_response(response: &tx_tracker::Response, tool: &str) {
        match tool {
            "create" => println!("Activity added!"),
            "list" => {
                if let Some(data) = &response.data {
                    if let Some(acts) = data.as_array() {
                        for act in acts {
                            let id = act["id"].as_i64().unwrap_or(0);
                            let start = act["start_time"].as_str().unwrap_or("");
                            let stop = act["stop_time"].as_str().unwrap_or("");
                            let desc = act["description"].as_str().unwrap_or("");
                            println!("[{}] {} -> {} : {}", id, start, stop, desc);
                        }
                    }
                }
            }
            "update" => println!("Activity updated!"),
            "delete" => println!("Activity deleted!"),
            _ => {}
        }
    }
}

#[cfg(feature = "cli")]
fn main() -> anyhow::Result<()> {
    cli::run()
}

#[cfg(not(feature = "cli"))]
fn main() {
    println!("CLI not enabled. Build with --features cli to enable.");
}
