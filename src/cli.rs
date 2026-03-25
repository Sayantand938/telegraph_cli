use clap::Parser;

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
}
