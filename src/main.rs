use clap::Parser;
use tx_tracker::{Request, Tracker};
use serde_json::json;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Domain: transaction / activity
    #[arg(long)]
    domain: String,

    /// Tool/action: create / list / update / delete
    #[arg(long)]
    tool: String,

    // Transaction fields
    #[arg(long)]
    amount: Option<f64>,
    #[arg(long)]
    kind: Option<String>,
    #[arg(long)]
    desc: Option<String>,
    #[arg(long)]
    id: Option<i64>,

    // Activity fields
    #[arg(long)]
    start: Option<String>,
    #[arg(long)]
    stop: Option<String>,
    #[arg(long)]
    activity_desc: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

fn build_request(cli: &Cli) -> anyhow::Result<Request> {
    let (tool, args) = match cli.domain.as_str() {
        "transaction" => match cli.tool.as_str() {
            "create" => {
                let amount = cli.amount.ok_or_else(|| anyhow::anyhow!("--amount is required"))?;
                let kind = cli.kind.clone().ok_or_else(|| anyhow::anyhow!("--kind is required"))?;
                let desc = cli.desc.clone().unwrap_or_default();
                ("create_transaction".to_string(), json!({
                    "amount": amount,
                    "kind": kind,
                    "description": desc
                }))
            }
            "list" => ("list_transactions".to_string(), json!({ "kind": cli.kind })),
            "update" => {
                let id = cli.id.ok_or_else(|| anyhow::anyhow!("--id is required"))?;
                ("update_transaction".to_string(), json!({
                    "id": id,
                    "amount": cli.amount,
                    "kind": cli.kind,
                    "description": cli.desc
                }))
            }
            "delete" => {
                let id = cli.id.ok_or_else(|| anyhow::anyhow!("--id is required"))?;
                ("delete_transaction".to_string(), json!({ "id": id }))
            }
            _ => return Err(anyhow::anyhow!("Unknown tool: {}", cli.tool)),
        },
        "activity" => match cli.tool.as_str() {
            "create" => {
                let start = cli.start.clone().ok_or_else(|| anyhow::anyhow!("--start is required"))?;
                let stop = cli.stop.clone().ok_or_else(|| anyhow::anyhow!("--stop is required"))?;
                let desc = cli.activity_desc.clone().unwrap_or_default();
                ("create_activity".to_string(), json!({
                    "start_time": start,
                    "stop_time": stop,
                    "description": desc
                }))
            }
            "list" => ("list_activities".to_string(), json!(null)),
            "update" | "delete" => return Err(anyhow::anyhow!("Activity update/delete not yet implemented")),
            _ => return Err(anyhow::anyhow!("Unknown tool: {}", cli.tool)),
        },
        _ => return Err(anyhow::anyhow!("Unknown domain: {}", cli.domain)),
    };

    Ok(Request { tool, args })
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
