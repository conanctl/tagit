use clap::Parser;
use crate::db;
use crate::utils::{now, resolve_path};
use crate::error::Result;
use super::args::{Cli, Commands};
use colored::*;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use shell_escape;

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

fn format_path_for_fzf(entry: &EnhancedPathEntry) -> String {
    let freq_indicator = format_frequency(entry.freq);
    let time_ago = format_duration(entry.last_used);
    let tags_display = if !entry.tags.is_empty() {
        format!(" [{}]", entry.tags.join(", "))
    } else {
        " [untagged]".to_string()
    };
    
    format!("{}{}{} {}", 
        entry.path,
        freq_indicator,
        tags_display,
        time_ago
    )
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

            Commands::Jump { pattern } => {
                let paths = db::list_paths(&conn)?;
                let mut enhanced_entries = Vec::new();
                let matcher = SkimMatcherV2::default();
                
                for path_entry in paths {
                    let resolved_path = resolve_path(Some(path_entry.path.clone()))?;
                    let path = Path::new(&resolved_path);
                    
                    if !path.exists() || !path.is_dir() {
                        continue;
                    }

                    let mut should_include = true;
                    let mut path_score = None;

                    if let Some(ref pattern) = pattern {
                        path_score = matcher.fuzzy_match(&resolved_path, pattern);
                        should_include = path_score.is_some();

                        if !should_include {
                            let tags = db::get_tags_for_path(&conn, path_entry.id.unwrap())?;
                            for tag in tags {
                                if let Some(score) = matcher.fuzzy_match(&tag, pattern) {
                                    should_include = true;
                                    path_score = Some(score);
                                    break;
                                }
                            }
                        }
                    }

                    if should_include {
                        let raw_tags = db::get_tags_for_path(&conn, path_entry.id.unwrap())?;
                        let tags: Vec<String> = raw_tags.iter()
                            .flat_map(|t| t.split(','))
                            .map(|s| s.trim().to_string())
                            .collect();

                        enhanced_entries.push(EnhancedPathEntry {
                            path: resolved_path,
                            tags,
                            last_used: path_entry.last_used,
                            freq: path_entry.freq,
                            score: path_score,
                        });
                    }
                }

                if enhanced_entries.is_empty() {
                    if let Some(ref p) = pattern {
                        eprintln!("{} {}", "No matching directories found for:".yellow(), p);
                    } else {
                        eprintln!("{}", "No tagged directories found".yellow());
                    }
                    println!(":");
                    return Ok(());
                }

                enhanced_entries.sort_by_key(|e| std::cmp::Reverse(e.freq));

                let entries: Vec<String> = enhanced_entries.iter()
                    .map(format_path_for_fzf)
                    .collect();

                let entries_str = entries.join("\n");
                
                let mut fzf = Command::new("fzf")
                    .arg("--ansi")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()?;

                if let Some(mut stdin) = fzf.stdin.take() {
                    use std::io::Write;
                    stdin.write_all(entries_str.as_bytes())?;
                }

                let output = fzf.wait_with_output()?;
                
                if output.status.success() {
                    if let Ok(selected) = String::from_utf8(output.stdout) {
                        if let Some(dir) = selected.split_whitespace().next() {
                            println!("cd {}", shell_escape::escape(dir.into()));
                            println!("echo '🚀 Jumped to {}'", shell_escape::escape(dir.into()));
                        } else {
                            println!(":");
                        }
                    }
                } else {
                    println!(":");
                }
            }

            Commands::Init { shell } => {
                match shell.as_str() {
                    "zsh" | "bash" => {
                        println!(r#"
function tag() {{
    if [ "$1" = "jump" ]; then
        local output
        output="$({} "$@")"
        if [ -n "$output" ]; then
            eval "$output"
        fi
    else
        {} "$@"
    fi
}}
"#, std::env::current_exe()?.display(), std::env::current_exe()?.display());
                    }
                    _ => {
                        eprintln!("Unsupported shell: {}", shell);
                        eprintln!("Currently supported shells: zsh, bash");
                    }
                }
            }
        }
        Ok(())
    }
} 