use clap::Parser;
use crate::db;
use crate::utils::{now, resolve_path, format_path_for_display};
use crate::error::Result;
use super::args::{Cli, Commands};
use colored::*;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
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

fn extract_path_from_fzf_selection(selection: &str) -> Option<String> {
    if let Some(tags_end) = selection.rfind("] ") {
        let before_time = &selection[..tags_end + 1];
        
        if let Some(tags_start) = before_time.rfind(" [") {
            let path_and_freq = &before_time[..tags_start];
            
            let path = path_and_freq
                .trim_end_matches(" ★★★")
                .trim_end_matches(" ★★")
                .trim_end_matches(" ★")
                .trim();
            
            return Some(path.to_string());
        }
    }
    
    let tokens: Vec<&str> = selection.split_whitespace().collect();
    if tokens.len() >= 3 {
        if let Some(last) = tokens.last() {
            if last.contains("ago") || *last == "now" {
                let path_part = tokens[..tokens.len()-1].join(" ");
                let path = path_part
                    .trim_end_matches(" ★★★")
                    .trim_end_matches(" ★★")
                    .trim_end_matches(" ★")
                    .trim();
                return Some(path.to_string());
            }
        }
    }
    
    selection.split_whitespace().next().map(|s| s.to_string())
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
    let star = |n: usize| {
        let s = "★".repeat(n);
        format!(" {}", s).truecolor(229,192,123).bold().to_string()
    };
    if freq < 5 {
        "".to_string()
    } else if freq < 10 {
        star(1)
    } else if freq < 20 {
        star(2)
    } else {
        star(3)
    }
}

fn format_path_for_fzf(entry: &EnhancedPathEntry) -> String {
    let freq_indicator = format_frequency(entry.freq);
    let time_ago = format_duration(entry.last_used);
    let tags_display = if !entry.tags.is_empty() {
        format!("\x1b[38;2;97;175;239m [{}]\x1b[0m", entry.tags.join(", "))
    } else {
        format!("\x1b[38;2;92;99;112m [untagged]\x1b[0m")
    };
    
    format!("\x1b[38;2;224;108;117;4m{}\x1b[0m{}{} \x1b[38;2;92;99;112;3m{}\x1b[0m", 
        format_path_for_display(&entry.path),
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
                    "✓".truecolor(152,195,121).bold(),
                    "Tagged".truecolor(152,195,121),
                    format_path_for_display(&resolved_path).truecolor(224,108,117).underline(),
                    format!("[{}]", message).truecolor(97,175,239)
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

                println!("{}", if pattern.is_some() { "Matches:" } else { "Paths:" }.truecolor(86,182,194).bold());
                for entry in &enhanced_entries {
                    let freq_indicator = format_frequency(entry.freq);
                    let time_ago = format_duration(entry.last_used);
                    let tags_display = if !entry.tags.is_empty() {
                        format!(" [{}]", entry.tags.join(", ")).truecolor(97,175,239).to_string()
                    } else {
                        " [untagged]".truecolor(92,99,112).to_string()
                    };
                    
                    println!("{} {} {}{} {}",
                        "•".truecolor(92,99,112),
                        format_path_for_display(&entry.path).truecolor(224,108,117).underline(),
                        freq_indicator,
                        tags_display,
                        time_ago.truecolor(92,99,112).italic()
                    );
                }
            }
            
            Commands::Rm { pattern, tags } => {
                let paths = db::list_paths(&conn)?;
                let mut enhanced_entries = Vec::new();
                let matcher = SkimMatcherV2::default();

                for path_entry in paths {
                    let mut should_include = pattern.is_none();
                    let mut path_score = None;

                    if let Some(ref p) = pattern {
                        path_score = matcher.fuzzy_match(&path_entry.path, p);
                        should_include = path_score.is_some();

                        if !should_include {
                            let path_tags = db::get_tags_for_path(&conn, path_entry.id.unwrap())?;
                            for tag in path_tags {
                                if let Some(score) = matcher.fuzzy_match(&tag, p) {
                                    should_include = true;
                                    path_score = Some(score);
                                    break;
                                }
                            }
                        }
                    }

                    if should_include {
                        let raw_tags = db::get_tags_for_path(&conn, path_entry.id.unwrap())?;
                        let entry_tags: Vec<String> = raw_tags.iter()
                            .flat_map(|t| t.split(','))
                            .map(|s| s.trim().to_string())
                            .collect();

                        enhanced_entries.push(EnhancedPathEntry {
                            path: path_entry.path,
                            tags: entry_tags,
                            last_used: path_entry.last_used,
                            freq: path_entry.freq,
                            score: path_score,
                        });
                    }
                }

                if enhanced_entries.is_empty() {
                    eprintln!("{}", "No paths found.".yellow());
                    return Ok(());
                }

                enhanced_entries.sort_by_key(|e| std::cmp::Reverse(e.score.unwrap_or(e.freq)));

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
                
                if !output.status.success() {
                    return Ok(());
                }

                if let Ok(selected) = String::from_utf8(output.stdout) {
                    if let Some(path_str) = extract_path_from_fzf_selection(&selected.trim()) {
                        let resolved_path = resolve_path(Some(path_str))?;
                        if tags.is_empty() {
                            db::remove_path(&mut conn, &resolved_path)?;
                            println!("{} {} {}",
                                "✓".green().bold(),
                                "Removed entry".green(),
                                format_path_for_display(&resolved_path).blue().underline()
                            );
                        } else {
                            db::remove_tags_from_path(&mut conn, &resolved_path, &tags)?;
                            println!("{} {} {} {}",
                                "✓".green().bold(),
                                "Removed tags from".green(),
                                format_path_for_display(&resolved_path).blue().underline(),
                                format!("[{}]", tags.join(", ")).yellow()
                            );
                        }
                    }
                }
            }

            Commands::Jump { pattern } => {
                if std::env::var("TAGIT_SHELL_INTEGRATION").is_err() {
                    eprintln!("{}", "Shell integration not found.".yellow());
                    eprintln!("For the jump command to work, you need to add the following to your shell configuration file (e.g., ~/.zshrc, ~/.bashrc):");
                    println!("\n{}", format!(r#"
function tag() {{
    if [ "$1" = "jump" ]; then
        local output
        output="$(TAGIT_SHELL_INTEGRATION=1 {} "$@")"
        if [[ -n "$output" && "$output" != ":" ]]; then
            eval "$output"
        fi
    else
        command {} "$@"
    fi
}}
"#, std::env::current_exe()?.display(), std::env::current_exe()?.display()).bright_black());
                    return Ok(());
                }

                let paths = db::list_paths(&conn)?;
                let mut enhanced_entries = Vec::new();
                let matcher = SkimMatcherV2::default();
                
                for path_entry in paths {
                    let path = Path::new(&path_entry.path);
                    let _ = path;

                    let mut should_include = true;
                    let mut path_score = None;

                    if let Some(ref pattern) = pattern {
                        path_score = matcher.fuzzy_match(&path_entry.path, pattern);
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
                        eprintln!("{} {}", "No matching paths found for:".yellow(), p);
                    } else {
                        eprintln!("{}", "No tagged paths found".yellow());
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
                    .arg("--color=fg:-1,bg:-1")
                    .arg("--color=hl:bright-red,fg+:-1,bg+:bright-black,hl+:bright-red")
                    .arg("--color=pointer:yellow,marker:yellow")
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
                        if let Some(selected_path) = extract_path_from_fzf_selection(&selected.trim()) {
                            let resolved_selected_path = if selected_path.starts_with('~') {
                                resolve_path(Some(selected_path.clone()))?
                            } else {
                                selected_path.clone()
                            };
                            
                            let path = Path::new(&resolved_selected_path);
                            let target_dir = if path.is_dir() {
                                resolved_selected_path.clone()
                            } else {
                                path.parent()
                                    .map(|p| p.to_string_lossy().to_string())
                                    .unwrap_or_else(|| resolved_selected_path.clone())
                            };
                            
                            println!("cd {}", shell_escape::escape(target_dir.clone().into()));
                            if path.is_file() {
                                println!("echo '🚀 Jumped to {} (parent of {})'", 
                                    shell_escape::escape(format_path_for_display(&target_dir).into()),
                                    shell_escape::escape(format_path_for_display(&resolved_selected_path).into())
                                );
                            } else {
                                println!("echo '🚀 Jumped to {}'", shell_escape::escape(format_path_for_display(&target_dir).into()));
                            }
                        } else {
                            println!(":");
                        }
                    }
                } else {
                    println!(":");
                }
            }
        }
        Ok(())
    }
} 