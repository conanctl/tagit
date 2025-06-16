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
    Add {
        #[arg(required = true)]
        path: String,

        #[arg(required = true)]
        message: String,
    },
    
    Ls {
        pattern: Option<String>,
    },
    
    Rm {
        tags: Option<String>,

        #[arg(short, long)]
        path: Option<String>,

        #[arg(short, long)]
        fuzzy: bool,
    },
} 