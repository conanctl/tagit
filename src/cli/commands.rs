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

        match &self.command {
            Commands::Add { path, message } => {
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
            
            Commands::Ls { pattern } => {
                let paths = db::list_paths(&conn)?;
                let mut enhanced_entries = Vec::new();
                let matcher = SkimMatcherV2::default();
                
                let matching_tag_paths = if let Some(ref pattern) = pattern {
                    let all_tags = db::list_all_tags(&conn)?;
                    let matching_tags: Vec<_> = all_tags.iter()
                        .filter_map(|t| {
                            matcher.fuzzy_match(t, pattern)
                                .map(|score| (t, score))
                        })
                        .collect();

                    let mut tag_paths = Vec::new();
                    for (tag, score) in matching_tags {
                        let paths = db::find_paths_by_tag(&conn, tag)?;
                        for path in paths {
                            tag_paths.push((path, score));
                        }
                    }
                    Some(tag_paths)
                } else {
                    None
                };

                for path_entry in paths {
                    let mut should_include = true;
                    let mut path_score = None;

                    if let Some(ref pattern) = pattern {
                        path_score = matcher.fuzzy_match(&path_entry.path, pattern);
                        should_include = path_score.is_some();
                    }

                    if let Some(ref tag_paths) = matching_tag_paths {
                        if let Some((_, tag_score)) = tag_paths.iter()
                            .find(|(p, _)| p.id == path_entry.id) {
                            should_include = true;
                            path_score = Some(*tag_score);
                        }
                    }

                    if should_include {
                        let raw_tags = db::get_tags_for_path(&conn, path_entry.id.unwrap())?;
                        let tags: Vec<String> = raw_tags.iter()
                            .flat_map(|t| t.split(','))
                            .map(|s| s.trim().to_string())
                            .collect();

                        enhanced_entries.push(EnhancedPathEntry {
                            path: path_entry.path,
                            tags,
                            last_used: path_entry.last_used,
                            freq: path_entry.freq,
                            score: path_score,
                        });
                    }
                }
                
                if enhanced_entries.is_empty() {
                    if let Some(ref p) = pattern {
                        println!("{} {}", "No matches found for:".yellow(), p);
                    } else {
                        println!("{}", "No paths found".yellow());
                    }
                    return Ok(());
                }

                if pattern.is_some() {
                    enhanced_entries.sort_by_key(|e| std::cmp::Reverse(e.score.unwrap_or(0)));
                } else {
                    enhanced_entries.sort_by_key(|e| std::cmp::Reverse(e.freq));
                }

                println!("{}", if pattern.is_some() { "Matches:" } else { "Paths:" }.green().bold());
                for entry in &enhanced_entries {
                    let freq_indicator = format_frequency(entry.freq);
                    let time_ago = format_duration(entry.last_used);
                    let tags_display = if !entry.tags.is_empty() {
                        format!(" [{}]", entry.tags.join(", ")).yellow().to_string()
                    } else {
                        " [untagged]".bright_black().to_string()
                    };
                    
                    println!("{} {} {}{} {}",
                        "•".bright_black(),
                        entry.path.blue().underline(),
                        freq_indicator,
                        tags_display,
                        time_ago.bright_black().italic()
                    );
                }
            }
            
            Commands::Rm { path, tags, fuzzy: _ } => {
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