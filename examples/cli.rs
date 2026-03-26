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
    /// Journal operations
    Journal {
        #[command(subcommand)]
        action: JournalAction,
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
        #[arg(short = 'a', long)]
        amount: f64,
        #[arg(short = 'k', long)]
        kind: String,
        #[arg(short = 'd', long, default_value = "")]
        desc: String,
        #[arg(long, visible_alias = "cat")]
        category: Option<String>,
        #[arg(long, visible_alias = "loc")]
        location: Option<String>,
        #[arg(long, visible_alias = "tag")]
        tags: Vec<String>,
        #[arg(long, visible_alias = "person")]
        people: Vec<String>,
    },
    /// Get a transaction by ID
    Get {
        #[arg(short = 'i', long)]
        id: i64,
    },
    /// List transactions
    List {
        #[arg(short = 'k', long)]
        kind: Option<String>,
        #[arg(long, visible_alias = "cat")]
        category: Option<String>,
        #[arg(long, visible_alias = "loc")]
        location: Option<String>,
    },
    /// Update a transaction
    Update {
        #[arg(short = 'i', long)]
        id: i64,
        #[arg(short = 'a', long)]
        amount: Option<f64>,
        #[arg(short = 'k', long)]
        kind: Option<String>,
        #[arg(short = 'd', long)]
        desc: Option<String>,
        #[arg(long, visible_alias = "cat")]
        category: Option<String>,
        #[arg(long, visible_alias = "loc")]
        location: Option<String>,
    },
    /// Delete a transaction
    Delete {
        #[arg(short = 'i', long)]
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
        #[arg(short = 'd', long, default_value = "")]
        desc: String,
        #[arg(long, visible_alias = "cat")]
        category: Option<String>,
        #[arg(long, visible_alias = "loc")]
        location: Option<String>,
        #[arg(long, visible_alias = "tag")]
        tags: Vec<String>,
        #[arg(long, visible_alias = "person")]
        people: Vec<String>,
    },
    /// Get an activity by ID
    Get {
        #[arg(short = 'i', long)]
        id: i64,
    },
    /// List activities
    List {
        #[arg(long, visible_alias = "cat")]
        category: Option<String>,
        #[arg(long, visible_alias = "loc")]
        location: Option<String>,
    },
    /// Update an activity
    Update {
        #[arg(short = 'i', long)]
        id: i64,
        #[arg(long)]
        start: Option<String>,
        #[arg(long)]
        stop: Option<String>,
        #[arg(short = 'd', long)]
        desc: Option<String>,
        #[arg(long, visible_alias = "cat")]
        category: Option<String>,
        #[arg(long, visible_alias = "loc")]
        location: Option<String>,
    },
    /// Delete an activity
    Delete {
        #[arg(short = 'i', long)]
        id: i64,
    },
}

#[derive(Subcommand)]
enum CategoryAction {
    /// List all categories
    List,
    /// Delete a category
    Delete {
        #[arg(short = 'i', long)]
        id: i64,
    },
}

#[derive(Subcommand)]
enum PlaceAction {
    /// List all places
    List,
    /// Delete a place
    Delete {
        #[arg(short = 'i', long)]
        id: i64,
    },
}

#[derive(Subcommand)]
enum TagAction {
    /// List all tags
    List,
    /// Delete a tag
    Delete {
        #[arg(short = 'i', long)]
        id: i64,
    },
}

#[derive(Subcommand)]
enum PersonAction {
    /// List all persons
    List,
    /// Delete a person
    Delete {
        #[arg(short = 'i', long)]
        id: i64,
    },
}

#[derive(Subcommand)]
enum TodoAction {
    /// Create a new todo
    Create {
        #[arg(short = 'd', long)]
        desc: String,
        #[arg(long, default_value = "pending")]
        status: String,
        #[arg(long, visible_alias = "prio")]
        priority: Option<String>,
        #[arg(long, visible_alias = "due")]
        due_date: Option<String>,
        #[arg(long, visible_alias = "cat")]
        category: Option<String>,
        #[arg(long, visible_alias = "loc")]
        location: Option<String>,
        #[arg(long, visible_alias = "tag")]
        tags: Vec<String>,
        #[arg(long, visible_alias = "person")]
        people: Vec<String>,
    },
    /// Get a todo by ID
    Get {
        #[arg(short = 'i', long)]
        id: i64,
    },
    /// List todos
    List {
        #[arg(long)]
        status: Option<String>,
        #[arg(long, visible_alias = "prio")]
        priority: Option<String>,
    },
    /// Update a todo
    Update {
        #[arg(short = 'i', long)]
        id: i64,
        #[arg(short = 'd', long)]
        desc: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long, visible_alias = "prio")]
        priority: Option<String>,
        #[arg(long, visible_alias = "due")]
        due_date: Option<String>,
        #[arg(long, visible_alias = "cat")]
        category: Option<String>,
        #[arg(long, visible_alias = "loc")]
        location: Option<String>,
    },
    /// Complete a todo
    Complete {
        #[arg(short = 'i', long)]
        id: i64,
    },
    /// Delete a todo
    Delete {
        #[arg(short = 'i', long)]
        id: i64,
    },
}

#[derive(Subcommand)]
enum JournalAction {
    /// Create a new journal entry
    Create {
        #[arg(long)]
        content: String,
        #[arg(short = 'd', long)]
        date: Option<String>,
        #[arg(long, visible_alias = "cat")]
        category: Option<String>,
        #[arg(long, visible_alias = "loc")]
        location: Option<String>,
        #[arg(long, visible_alias = "tag")]
        tags: Vec<String>,
        #[arg(long, visible_alias = "person")]
        people: Vec<String>,
    },
    /// Get a journal entry by ID
    Get {
        #[arg(short = 'i', long)]
        id: i64,
    },
    /// List journal entries
    List {
        #[arg(long, visible_alias = "from")]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
        #[arg(long, visible_alias = "cat")]
        category: Option<String>,
    },
    /// Search journal entries (full-text search)
    Search {
        #[arg(short = 'q', long)]
        query: String,
        #[arg(long, visible_alias = "from")]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
    },
    /// Update a journal entry
    Update {
        #[arg(short = 'i', long)]
        id: i64,
        #[arg(long)]
        content: Option<String>,
        #[arg(short = 'd', long)]
        date: Option<String>,
        #[arg(long, visible_alias = "cat")]
        category: Option<String>,
        #[arg(long, visible_alias = "loc")]
        location: Option<String>,
    },
    /// Delete a journal entry
    Delete {
        #[arg(short = 'i', long)]
        id: i64,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let tracker = Tracker::new(None).await?;

    match cli.command {
        Commands::Transaction { action } => match action {
            TransactionAction::Create { amount, kind, desc, category, location, tags, people } => {
                let mut args = json!({
                    "amount": amount,
                    "kind": kind,
                    "description": desc
                });
                if let Some(cat) = category { args["category"] = json!(cat); }
                if let Some(l) = location { args["location"] = json!(l); }
                if !tags.is_empty() { args["tags"] = json!(tags); }
                if !people.is_empty() { args["people"] = json!(people); }

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
            TransactionAction::List { kind, category: _, location: _ } => {
                let mut args = json!({});
                if let Some(k) = kind { args["kind"] = json!(k); }
                // Note: category/location filtering by ID would need lookup first

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
            TransactionAction::Update { id, amount, kind, desc, category, location } => {
                let mut args = json!({"id": id});
                if let Some(a) = amount { args["amount"] = json!(a); }
                if let Some(k) = kind { args["kind"] = json!(k); }
                if let Some(d) = desc { args["description"] = json!(d); }
                if let Some(c) = category { args["category"] = json!(c); }
                if let Some(l) = location { args["location"] = json!(l); }

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
            ActivityAction::Create { start, stop, desc, category, location, tags, people } => {
                let mut args = json!({
                    "start_time": start,
                    "stop_time": stop,
                    "description": desc
                });
                if let Some(cat) = category { args["category"] = json!(cat); }
                if let Some(l) = location { args["location"] = json!(l); }
                if !tags.is_empty() { args["tags"] = json!(tags); }
                if !people.is_empty() { args["people"] = json!(people); }

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
            ActivityAction::List { category: _, location: _ } => {
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
            ActivityAction::Update { id, start, stop, desc, category, location } => {
                let mut args = json!({"id": id});
                if let Some(s) = start { args["start_time"] = json!(s); }
                if let Some(s) = stop { args["stop_time"] = json!(s); }
                if let Some(d) = desc { args["description"] = json!(d); }
                if let Some(c) = category { args["category"] = json!(c); }
                if let Some(l) = location { args["location"] = json!(l); }

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
            TodoAction::Create { desc, status, priority, due_date, category, location, tags, people } => {
                let mut args = json!({
                    "description": desc,
                    "status": status
                });
                if let Some(p) = priority { args["priority"] = json!(p); }
                if let Some(d) = due_date { args["due_date"] = json!(d); }
                if let Some(c) = category { args["category"] = json!(c); }
                if let Some(l) = location { args["location"] = json!(l); }
                if !tags.is_empty() { args["tags"] = json!(tags); }
                if !people.is_empty() { args["people"] = json!(people); }

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
            TodoAction::Update { id, desc, status, priority, due_date, category, location } => {
                let mut args = json!({"id": id});
                if let Some(d) = desc { args["description"] = json!(d); }
                if let Some(s) = status { args["status"] = json!(s); }
                if let Some(p) = priority { args["priority"] = json!(p); }
                if let Some(d) = due_date { args["due_date"] = json!(d); }
                if let Some(c) = category { args["category"] = json!(c); }
                if let Some(l) = location { args["location"] = json!(l); }

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

        Commands::Journal { action } => match action {
            JournalAction::Create { content, date, category, location, tags, people } => {
                let mut args = json!({
                    "content": content,
                });
                if let Some(d) = date { args["date"] = json!(d); }
                if let Some(c) = category { args["category"] = json!(c); }
                if let Some(l) = location { args["location"] = json!(l); }
                if !tags.is_empty() { args["tags"] = json!(tags); }
                if !people.is_empty() { args["people"] = json!(people); }

                let resp = tracker.handle(&logbook::Request {
                    tool: "create_journal".into(),
                    args,
                }).await;

                if resp.success {
                    println!("✓ {}", resp.message.unwrap_or_default());
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
            JournalAction::Get { id } => {
                let resp = tracker.handle(&logbook::Request {
                    tool: "get_journal".into(),
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
            JournalAction::List { from, to, category } => {
                let mut args = json!({});
                if let Some(f) = from { args["from"] = json!(f); }
                if let Some(t) = to { args["to"] = json!(t); }
                if let Some(c) = category { args["category"] = json!(c); }

                let resp = tracker.handle(&logbook::Request {
                    tool: "list_journals".into(),
                    args,
                }).await;

                if resp.success {
                    if let Some(data) = resp.data {
                        if let Some(arr) = data.as_array() {
                            for item in arr {
                                let id = item["id"].as_i64().unwrap_or(0);
                                let date = item["date"].as_str().unwrap_or("-");
                                let content = item["content"].as_str().unwrap_or("");
                                let truncated = if content.len() > 50 {
                                    format!("{}...", &content[..50])
                                } else {
                                    content.to_string()
                                };
                                println!("[{}] ({}) {}", id, date, truncated);
                            }
                        }
                    }
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
            JournalAction::Search { query, from, to } => {
                let mut args = json!({
                    "query": query,
                });
                if let Some(f) = from { args["from"] = json!(f); }
                if let Some(t) = to { args["to"] = json!(t); }

                let resp = tracker.handle(&logbook::Request {
                    tool: "search_journals".into(),
                    args,
                }).await;

                if resp.success {
                    if let Some(data) = resp.data {
                        if let Some(arr) = data.as_array() {
                            for item in arr {
                                let id = item["id"].as_i64().unwrap_or(0);
                                let date = item["date"].as_str().unwrap_or("-");
                                let content = item["content"].as_str().unwrap_or("");
                                let truncated = if content.len() > 50 {
                                    format!("{}...", &content[..50])
                                } else {
                                    content.to_string()
                                };
                                println!("[{}] ({}) {}", id, date, truncated);
                            }
                        }
                    }
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
            JournalAction::Update { id, content, date, category, location } => {
                let mut args = json!({"id": id});
                if let Some(c) = content { args["content"] = json!(c); }
                if let Some(d) = date { args["date"] = json!(d); }
                if let Some(c) = category { args["category"] = json!(c); }
                if let Some(l) = location { args["location"] = json!(l); }

                let resp = tracker.handle(&logbook::Request {
                    tool: "update_journal".into(),
                    args,
                }).await;

                if resp.success {
                    println!("✓ {}", resp.message.unwrap_or_default());
                } else {
                    eprintln!("✗ Error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
            }
            JournalAction::Delete { id } => {
                let resp = tracker.handle(&logbook::Request {
                    tool: "delete_journal".into(),
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
