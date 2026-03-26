//! CLI for tx_tracker - Testing/Development Only
//!
//! Usage: cargo run --example cli --features cli -- [OPTIONS]
//!
//! Examples:
//!   cargo run --example cli -- --domain transaction --tool list
//!   cargo run --example cli -- --domain activity --tool create --start "10:00" --stop "12:00" --activity-desc "Meeting"

use clap::{Parser, Subcommand};
use logbook::Tracker;
use serde_json::json;

#[derive(Parser)]
#[command(author, version, about = "logbook CLI - For testing only")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Transaction operations
    Transaction {
        #[command(subcommand)]
        action: TransactionAction,
    },
    /// Activity operations
    Activity {
        #[command(subcommand)]
        action: ActivityAction,
    },
    /// Todo operations
    Todo {
        #[command(subcommand)]
        action: TodoAction,
    },
    /// Category operations
    Category {
        #[command(subcommand)]
        action: CategoryAction,
    },
    /// Place operations
    Place {
        #[command(subcommand)]
        action: PlaceAction,
    },
    /// Tag operations
    Tag {
        #[command(subcommand)]
        action: TagAction,
    },
    /// Person operations
    Person {
        #[command(subcommand)]
        action: PersonAction,
    },
}

#[derive(Subcommand)]
enum TransactionAction {
    /// Create a new transaction
    Create {
        #[arg(long)]
        amount: f64,
        #[arg(long)]
        kind: String,
        #[arg(long, default_value = "")]
        desc: String,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        place: Option<String>,
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        persons: Vec<String>,
    },
    /// Get a transaction by ID
    Get {
        #[arg(long)]
        id: i64,
    },
    /// List transactions
    List {
        #[arg(long)]
        kind: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        place: Option<String>,
    },
    /// Update a transaction
    Update {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        amount: Option<f64>,
        #[arg(long)]
        kind: Option<String>,
        #[arg(long)]
        desc: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        place: Option<String>,
    },
    /// Delete a transaction
    Delete {
        #[arg(long)]
        id: i64,
    },
}

#[derive(Subcommand)]
enum ActivityAction {
    /// Create a new activity
    Create {
        #[arg(long)]
        start: String,
        #[arg(long)]
        stop: String,
        #[arg(long, default_value = "")]
        desc: String,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        place: Option<String>,
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        persons: Vec<String>,
    },
    /// Get an activity by ID
    Get {
        #[arg(long)]
        id: i64,
    },
    /// List activities
    List {
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        place: Option<String>,
    },
    /// Update an activity
    Update {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        start: Option<String>,
        #[arg(long)]
        stop: Option<String>,
        #[arg(long)]
        desc: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        place: Option<String>,
    },
    /// Delete an activity
    Delete {
        #[arg(long)]
        id: i64,
    },
}

#[derive(Subcommand)]
enum CategoryAction {
    /// List all categories
    List,
    /// Delete a category
    Delete {
        #[arg(long)]
        id: i64,
    },
}

#[derive(Subcommand)]
enum PlaceAction {
    /// List all places
    List,
    /// Delete a place
    Delete {
        #[arg(long)]
        id: i64,
    },
}

#[derive(Subcommand)]
enum TagAction {
    /// List all tags
    List,
    /// Delete a tag
    Delete {
        #[arg(long)]
        id: i64,
    },
}

#[derive(Subcommand)]
enum PersonAction {
    /// List all persons
    List,
    /// Delete a person
    Delete {
        #[arg(long)]
        id: i64,
    },
}

#[derive(Subcommand)]
enum TodoAction {
    /// Create a new todo
    Create {
        #[arg(long)]
        desc: String,
        #[arg(long, default_value = "pending")]
        status: String,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        due_date: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        place: Option<String>,
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        persons: Vec<String>,
    },
    /// Get a todo by ID
    Get {
        #[arg(long)]
        id: i64,
    },
    /// List todos
    List {
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        priority: Option<String>,
    },
    /// Update a todo
    Update {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        desc: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        due_date: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        place: Option<String>,
    },
    /// Complete a todo
    Complete {
        #[arg(long)]
        id: i64,
    },
    /// Delete a todo
    Delete {
        #[arg(long)]
        id: i64,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let tracker = Tracker::new(None).await?;

    match cli.command {
        Commands::Transaction { action } => match action {
            TransactionAction::Create { amount, kind, desc, category, place, tags, persons } => {
                let mut args = json!({
                    "amount": amount,
                    "kind": kind,
                    "description": desc
                });
                if let Some(cat) = category { args["category"] = json!(cat); }
                if let Some(p) = place { args["place"] = json!(p); }
                if !tags.is_empty() { args["tags"] = json!(tags); }
                if !persons.is_empty() { args["persons"] = json!(persons); }

                let resp = tracker.handle(&logbook::Request {
                    tool: "create_transaction".into(),
                    args,
                }).await;

                if resp.success {
                    println!("✓ {}", resp.message.unwrap_or_default());
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
            TransactionAction::Get { id } => {
                let resp = tracker.handle(&logbook::Request {
                    tool: "get_transaction".into(),
                    args: json!({"id": id}),
                }).await;

                if resp.success {
                    if let Some(data) = resp.data {
                        println!("{}", serde_json::to_string_pretty(&data)?);
                    }
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
            TransactionAction::List { kind, category: _, place: _ } => {
                let mut args = json!({});
                if let Some(k) = kind { args["kind"] = json!(k); }
                // Note: category/place filtering by ID would need lookup first

                let resp = tracker.handle(&logbook::Request {
                    tool: "list_transactions".into(),
                    args,
                }).await;

                if resp.success {
                    if let Some(data) = resp.data {
                        if let Some(arr) = data.as_array() {
                            for item in arr {
                                let id = item["id"].as_i64().unwrap_or(0);
                                let kind = item["kind"].as_str().unwrap_or("");
                                let desc = item["description"].as_str().unwrap_or("");
                                let amount = item["amount"].as_f64().unwrap_or(0.0);
                                println!("[{}] {} - {} ({})", id, kind, desc, amount);
                            }
                        }
                    }
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
            TransactionAction::Update { id, amount, kind, desc, category, place } => {
                let mut args = json!({"id": id});
                if let Some(a) = amount { args["amount"] = json!(a); }
                if let Some(k) = kind { args["kind"] = json!(k); }
                if let Some(d) = desc { args["description"] = json!(d); }
                if let Some(c) = category { args["category"] = json!(c); }
                if let Some(p) = place { args["place"] = json!(p); }

                let resp = tracker.handle(&logbook::Request {
                    tool: "update_transaction".into(),
                    args,
                }).await;

                if resp.success {
                    println!("✓ {}", resp.message.unwrap_or_default());
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
            TransactionAction::Delete { id } => {
                let resp = tracker.handle(&logbook::Request {
                    tool: "delete_transaction".into(),
                    args: json!({"id": id}),
                }).await;

                if resp.success {
                    println!("✓ {}", resp.message.unwrap_or_default());
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
        },

        Commands::Activity { action } => match action {
            ActivityAction::Create { start, stop, desc, category, place, tags, persons } => {
                let mut args = json!({
                    "start_time": start,
                    "stop_time": stop,
                    "description": desc
                });
                if let Some(cat) = category { args["category"] = json!(cat); }
                if let Some(p) = place { args["place"] = json!(p); }
                if !tags.is_empty() { args["tags"] = json!(tags); }
                if !persons.is_empty() { args["persons"] = json!(persons); }

                let resp = tracker.handle(&logbook::Request {
                    tool: "create_activity".into(),
                    args,
                }).await;

                if resp.success {
                    println!("✓ {}", resp.message.unwrap_or_default());
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
            ActivityAction::Get { id } => {
                let resp = tracker.handle(&logbook::Request {
                    tool: "get_activity".into(),
                    args: json!({"id": id}),
                }).await;

                if resp.success {
                    if let Some(data) = resp.data {
                        println!("{}", serde_json::to_string_pretty(&data)?);
                    }
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
            ActivityAction::List { category: _, place: _ } => {
                let resp = tracker.handle(&logbook::Request {
                    tool: "list_activities".into(),
                    args: json!({}),
                }).await;

                if resp.success {
                    if let Some(data) = resp.data {
                        if let Some(arr) = data.as_array() {
                            for item in arr {
                                let id = item["id"].as_i64().unwrap_or(0);
                                let start = item["start_time"].as_str().unwrap_or("");
                                let stop = item["stop_time"].as_str().unwrap_or("");
                                let desc = item["description"].as_str().unwrap_or("");
                                println!("[{}] {} -> {} : {}", id, start, stop, desc);
                            }
                        }
                    }
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
            ActivityAction::Update { id, start, stop, desc, category, place } => {
                let mut args = json!({"id": id});
                if let Some(s) = start { args["start_time"] = json!(s); }
                if let Some(s) = stop { args["stop_time"] = json!(s); }
                if let Some(d) = desc { args["description"] = json!(d); }
                if let Some(c) = category { args["category"] = json!(c); }
                if let Some(p) = place { args["place"] = json!(p); }

                let resp = tracker.handle(&logbook::Request {
                    tool: "update_activity".into(),
                    args,
                }).await;

                if resp.success {
                    println!("✓ {}", resp.message.unwrap_or_default());
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
            ActivityAction::Delete { id } => {
                let resp = tracker.handle(&logbook::Request {
                    tool: "delete_activity".into(),
                    args: json!({"id": id}),
                }).await;

                if resp.success {
                    println!("✓ {}", resp.message.unwrap_or_default());
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
        },

        Commands::Category { action } => match action {
            CategoryAction::List => {
                let resp = tracker.handle(&logbook::Request {
                    tool: "list_categories".into(),
                    args: json!({}),
                }).await;

                if resp.success {
                    if let Some(data) = resp.data {
                        if let Some(arr) = data.as_array() {
                            for item in arr {
                                let id = item["id"].as_i64().unwrap_or(0);
                                let name = item["name"].as_str().unwrap_or("");
                                println!("[{}] {}", id, name);
                            }
                        }
                    }
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
            CategoryAction::Delete { id } => {
                let resp = tracker.handle(&logbook::Request {
                    tool: "delete_category".into(),
                    args: json!({"id": id}),
                }).await;

                if resp.success {
                    println!("✓ {}", resp.message.unwrap_or_default());
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
        },

        Commands::Place { action } => match action {
            PlaceAction::List => {
                let resp = tracker.handle(&logbook::Request {
                    tool: "list_places".into(),
                    args: json!({}),
                }).await;

                if resp.success {
                    if let Some(data) = resp.data {
                        if let Some(arr) = data.as_array() {
                            for item in arr {
                                let id = item["id"].as_i64().unwrap_or(0);
                                let name = item["name"].as_str().unwrap_or("");
                                println!("[{}] {}", id, name);
                            }
                        }
                    }
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
            PlaceAction::Delete { id } => {
                let resp = tracker.handle(&logbook::Request {
                    tool: "delete_place".into(),
                    args: json!({"id": id}),
                }).await;

                if resp.success {
                    println!("✓ {}", resp.message.unwrap_or_default());
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
        },

        Commands::Tag { action } => match action {
            TagAction::List => {
                let resp = tracker.handle(&logbook::Request {
                    tool: "list_tags".into(),
                    args: json!({}),
                }).await;

                if resp.success {
                    if let Some(data) = resp.data {
                        if let Some(arr) = data.as_array() {
                            for item in arr {
                                let id = item["id"].as_i64().unwrap_or(0);
                                let name = item["name"].as_str().unwrap_or("");
                                println!("[{}] {}", id, name);
                            }
                        }
                    }
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
            TagAction::Delete { id } => {
                let resp = tracker.handle(&logbook::Request {
                    tool: "delete_tag".into(),
                    args: json!({"id": id}),
                }).await;

                if resp.success {
                    println!("✓ {}", resp.message.unwrap_or_default());
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
        },

        Commands::Todo { action } => match action {
            TodoAction::Create { desc, status, priority, due_date, category, place, tags, persons } => {
                let mut args = json!({
                    "description": desc,
                    "status": status
                });
                if let Some(p) = priority { args["priority"] = json!(p); }
                if let Some(d) = due_date { args["due_date"] = json!(d); }
                if let Some(c) = category { args["category"] = json!(c); }
                if let Some(p) = place { args["place"] = json!(p); }
                if !tags.is_empty() { args["tags"] = json!(tags); }
                if !persons.is_empty() { args["persons"] = json!(persons); }

                let resp = tracker.handle(&logbook::Request {
                    tool: "create_todo".into(),
                    args,
                }).await;

                if resp.success {
                    println!("✓ {}", resp.message.unwrap_or_default());
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
            TodoAction::Get { id } => {
                let resp = tracker.handle(&logbook::Request {
                    tool: "get_todo".into(),
                    args: json!({"id": id}),
                }).await;

                if resp.success {
                    if let Some(data) = resp.data {
                        println!("{}", serde_json::to_string_pretty(&data)?);
                    }
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
            TodoAction::List { status, priority } => {
                let mut args = json!({});
                if let Some(s) = status { args["status"] = json!(s); }
                if let Some(p) = priority { args["priority"] = json!(p); }

                let resp = tracker.handle(&logbook::Request {
                    tool: "list_todos".into(),
                    args,
                }).await;

                if resp.success {
                    if let Some(data) = resp.data {
                        if let Some(arr) = data.as_array() {
                            for item in arr {
                                let id = item["id"].as_i64().unwrap_or(0);
                                let desc = item["description"].as_str().unwrap_or("");
                                let status = item["status"].as_str().unwrap_or("");
                                let priority = item["priority"].as_str().unwrap_or("-");
                                let due = item["due_date"].as_str().unwrap_or("-");
                                println!("[{}] {} - {} (priority: {}, due: {})", id, desc, status, priority, due);
                            }
                        }
                    }
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
            TodoAction::Update { id, desc, status, priority, due_date, category, place } => {
                let mut args = json!({"id": id});
                if let Some(d) = desc { args["description"] = json!(d); }
                if let Some(s) = status { args["status"] = json!(s); }
                if let Some(p) = priority { args["priority"] = json!(p); }
                if let Some(d) = due_date { args["due_date"] = json!(d); }
                if let Some(c) = category { args["category"] = json!(c); }
                if let Some(p) = place { args["place"] = json!(p); }

                let resp = tracker.handle(&logbook::Request {
                    tool: "update_todo".into(),
                    args,
                }).await;

                if resp.success {
                    println!("✓ {}", resp.message.unwrap_or_default());
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
            TodoAction::Complete { id } => {
                let resp = tracker.handle(&logbook::Request {
                    tool: "complete_todo".into(),
                    args: json!({"id": id}),
                }).await;

                if resp.success {
                    println!("✓ {}", resp.message.unwrap_or_default());
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
            TodoAction::Delete { id } => {
                let resp = tracker.handle(&logbook::Request {
                    tool: "delete_todo".into(),
                    args: json!({"id": id}),
                }).await;

                if resp.success {
                    println!("✓ {}", resp.message.unwrap_or_default());
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
        },

        Commands::Person { action } => match action {
            PersonAction::List => {
                let resp = tracker.handle(&logbook::Request {
                    tool: "list_persons".into(),
                    args: json!({}),
                }).await;

                if resp.success {
                    if let Some(data) = resp.data {
                        if let Some(arr) = data.as_array() {
                            for item in arr {
                                let id = item["id"].as_i64().unwrap_or(0);
                                let name = item["name"].as_str().unwrap_or("");
                                println!("[{}] {}", id, name);
                            }
                        }
                    }
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
            PersonAction::Delete { id } => {
                let resp = tracker.handle(&logbook::Request {
                    tool: "delete_person".into(),
                    args: json!({"id": id}),
                }).await;

                if resp.success {
                    println!("✓ {}", resp.message.unwrap_or_default());
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
        },
    }

    Ok(())
}
