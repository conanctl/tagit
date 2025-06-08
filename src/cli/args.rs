use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "pb")]
#[command(about = "A frictionless path management tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Tag {
        #[arg(required = true)]
        tags: Vec<String>,

        #[arg(short, long)]
        path: Option<String>,
    },
    
    #[command(alias = "ls")]
    List {
        pattern: Option<String>,

        #[arg(short, long)]
        fuzzy: bool,
    },
    
    #[command(alias = "find")]
    Search {
        tag: String,

        #[arg(short, long)]
        fuzzy: bool,

        #[arg(short, long)]
        scores: bool,
    },
    
    #[command(alias = "untag")]
    Remove {
        tags: Vec<String>,

        #[arg(short, long)]
        path: Option<String>,

        #[arg(short, long)]
        fuzzy: bool,
    },
} 