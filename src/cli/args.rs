use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "tag")]
#[command(about = "A frictionless file tagging tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Tag {
        #[arg(required = true)]
        path: String,

        #[arg(required = true)]
        message: String,
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
        scores: bool,
    },
    
    #[command(alias = "untag")]
    Remove {
        tags: Option<String>,

        #[arg(short, long)]
        path: Option<String>,

        #[arg(short, long)]
        fuzzy: bool,
    },
} 