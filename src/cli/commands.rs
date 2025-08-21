use clap::Parser;
use crate::db;
use crate::utils::{now, resolve_path};
use crate::error::Result;
use super::args::{Cli, Commands};
use colored::*;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

struct EnhancedPathEntry {
    path: String,
    tags: Vec<String>,
    last_used: i64,
    freq: i64,
    score: Option<i64>,
}

fn format_duration(timestamp: i64) -> String {
    let now = now();
    let duration = now - timestamp;
    
    if duration < 60 {
        "just now".to_string()
    } else if duration < 3600 {
        format!("{} minutes ago", duration / 60)
    } else if duration < 86400 {
        format!("{} hours ago", duration / 3600)
    } else {
        format!("{} days ago", duration / 86400)
    }
}

fn format_frequency(freq: i64) -> String {
    if freq < 5 {
        "".to_string()
    } else if freq < 10 {
        " ★".yellow().bold().to_string()
    } else if freq < 20 {
        " ★★".yellow().bold().to_string()
    } else {
        " ★★★".yellow().bold().to_string()
    }
}

pub fn run_cli() -> Result<()> {
    let cli = Cli::parse();
    cli.run()
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        let mut conn = db::open_db()?;
        let matcher = SkimMatcherV2::default();

        match &self.command {
            Commands::Tag { path, message } => {
                let resolved_path = resolve_path(Some(path.clone()))?;
                let tags_vec = vec![message.clone()];
                db::create_path_tag_entry(&mut conn, &resolved_path, &tags_vec, now())?;
                println!("{} {} {} {}", 
                    "✓".green().bold(),
                    "Tagged".green(),
                    resolved_path.blue().underline(),
                    format!("[{}]", message).yellow()
                );
            }
            
            Commands::List { pattern, fuzzy } => {
                let paths = db::list_paths(&conn)?;
                let mut enhanced_entries = Vec::new();
                
                for path_entry in paths {
                    let should_include = if let Some(ref pattern) = pattern {
                        if *fuzzy {
                            matcher.fuzzy_match(&path_entry.path, pattern).is_some()
                        } else {
                            path_entry.path.contains(pattern)
                        }
                    } else {
                        true
                    };

                    if should_include {
                        let raw_tags = db::get_tags_for_path(&conn, path_entry.id.unwrap())?;
                        let tags: Vec<String> = raw_tags.iter()
                            .flat_map(|t| t.split(','))
                            .map(|s| s.trim().to_string())
                            .collect();
                        
                        let score = if *fuzzy {
                            pattern.as_ref()
                                .and_then(|p| matcher.fuzzy_match(&path_entry.path, p))
                        } else {
                            None
                        };

                        enhanced_entries.push(EnhancedPathEntry {
                            path: path_entry.path,
                            tags,
                            last_used: path_entry.last_used,
                            freq: path_entry.freq,
                            score,
                        });
                    }
                }
                
                if enhanced_entries.is_empty() {
                    println!("{}", "No paths found".yellow());
                    return Ok(());
                }

                if *fuzzy && pattern.is_some() {
                    enhanced_entries.sort_by_key(|e| std::cmp::Reverse(e.score.unwrap_or(0)));
                } else {
                    enhanced_entries.sort_by_key(|e| std::cmp::Reverse(e.freq));
                }

                println!("{}", "Paths:".green().bold());
                for entry in &enhanced_entries {
                    let freq_indicator = format_frequency(entry.freq);
                    let time_ago = format_duration(entry.last_used);
                    let tags_display = if !entry.tags.is_empty() {
                        format!(" [{}]", entry.tags.join(", ")).yellow().to_string()
                    } else {
                        " [untagged]".bright_black().to_string()
                    };
                    let score_display = if *fuzzy && pattern.is_some() {
                        format!(" (score: {})", entry.score.unwrap_or(0))
                    } else {
                        String::new()
                    };
                    
                    println!("{} {} {}{} {} {}",
                        "•".bright_black(),
                        entry.path.blue().underline(),
                        freq_indicator,
                        tags_display,
                        time_ago.bright_black().italic(),
                        score_display.bright_black()
                    );
                }
            }
            
            Commands::Search { tag, scores } => {
                let all_tags = db::list_all_tags(&conn)?;
                let matching_tags: Vec<_> = all_tags.iter()
                    .filter_map(|t| {
                        matcher.fuzzy_match(t, tag)
                            .map(|score| (t, score))
                    })
                    .collect();

                if matching_tags.is_empty() {
                    println!("{}", "No matching tags found".yellow());
                    return Ok(());
                }

                let mut all_paths = Vec::new();
                for (tag, score) in matching_tags {
                    let paths = db::find_paths_by_tag(&conn, tag)?;
                    for path in paths {
                        let tags = db::get_tags_for_path(&conn, path.id.unwrap())?;
                        all_paths.push((path, tags, score));
                    }
                }

                all_paths.sort_by_key(|(path, _, score)| (-score, -path.freq));

                for (path, tags, score) in all_paths {
                    let freq_indicator = format_frequency(path.freq);
                    let time_ago = format_duration(path.last_used);
                    let score_display = if *scores {
                        format!(" (score: {})", score)
                    } else {
                        String::new()
                    };
                    
                    println!("{} {} {}{} {} {}",
                        "•".bright_black(),
                        path.path.blue().underline(),
                        freq_indicator,
                        format!(" [{}]", tags.join(", ")).yellow(),
                        time_ago.bright_black().italic(),
                        score_display.bright_black()
                    );
                }
            }

            Commands::Remove { path, tags, fuzzy: _ } => {
                let resolved_path = resolve_path(path.clone())?;
                if let Some(tags) = tags {
                    if !tags.is_empty() {
                        let tags_vec = vec![tags.clone()];
                        db::remove_tags_from_path(&mut conn, &resolved_path, &tags_vec)?;
                        println!("{} {} {} {}",
                            "✓".green().bold(),
                            "Removed tags from".green(),
                            resolved_path.blue().underline(),
                            format!("[{}]", tags).yellow()
                        );
                        return Ok(());
                    }
                }
                db::remove_all_tags_from_path(&mut conn, &resolved_path)?;
                println!("{} {} {}",
                    "✓".green().bold(),
                    "Removed all tags from".green(),
                    resolved_path.blue().underline()
                );
            }
        }
        Ok(())
    }
} 