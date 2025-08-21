use pathbrain::cli::{Cli, Commands};
use clap::Parser;

fn parse_args(args: &[&str]) -> Cli {
    Cli::try_parse_from(args).unwrap()
}

#[test]
fn test_tag_command() {
    let cli = parse_args(&["pb", "tag", "/test/path", "need to implement caching"]);
    match cli.command {
        Commands::Tag { path, message } => {
            assert_eq!(path, "/test/path");
            assert_eq!(message, "need to implement caching");
        }
        _ => panic!("Expected Tag command"),
    }

    let cli = parse_args(&["pb", "tag", ".", "work in progress"]);
    match cli.command {
        Commands::Tag { path, message } => {
            assert_eq!(path, ".");
            assert_eq!(message, "work in progress");
        }
        _ => panic!("Expected Tag command"),
    }
}

#[test]
fn test_list_command() {
    let cli = parse_args(&["pb", "list"]);
    match cli.command {
        Commands::List { pattern, fuzzy } => {
            assert!(pattern.is_none());
            assert!(!fuzzy);
        }
        _ => panic!("Expected List command"),
    }

    let cli = parse_args(&["pb", "list", "filter"]);
    match cli.command {
        Commands::List { pattern, fuzzy } => {
            assert_eq!(pattern.unwrap(), "filter");
            assert!(!fuzzy);
        }
        _ => panic!("Expected List command"),
    }

    let cli = parse_args(&["pb", "ls", "filter"]);
    match cli.command {
        Commands::List { pattern, fuzzy } => {
            assert_eq!(pattern.unwrap(), "filter");
            assert!(!fuzzy);
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_search_command() {
    let cli = parse_args(&["pb", "search", "searchtag"]);
    match cli.command {
        Commands::Search { tag, scores } => {
            assert_eq!(tag, "searchtag");
            assert!(!scores);
        }
        _ => panic!("Expected Search command"),
    }

    let cli = parse_args(&["pb", "find", "searchtag"]);
    match cli.command {
        Commands::Search { tag, scores } => {
            assert_eq!(tag, "searchtag");
            assert!(!scores);
        }
        _ => panic!("Expected Search command"),
    }
}

#[test]
fn test_remove_command() {
    let cli = parse_args(&["pb", "remove", "need to implement caching", "-p", "/test/path"]);
    match cli.command {
        Commands::Remove { path, tags, fuzzy } => {
            assert_eq!(path, Some("/test/path".to_string()));
            assert_eq!(tags, Some("need to implement caching".to_string()));
            assert!(!fuzzy);
        }
        _ => panic!("Expected Remove command"),
    }

    let cli = parse_args(&["pb", "remove"]);
    match cli.command {
        Commands::Remove { path, tags, fuzzy } => {
            assert!(path.is_none());
            assert!(tags.is_none());
            assert!(!fuzzy);
        }
        _ => panic!("Expected Remove command"),
    }

    let cli = parse_args(&["pb", "untag", "work in progress", "--path", "/test/path"]);
    match cli.command {
        Commands::Remove { path, tags, fuzzy } => {
            assert_eq!(path, Some("/test/path".to_string()));
            assert_eq!(tags, Some("work in progress".to_string()));
            assert!(!fuzzy);
        }
        _ => panic!("Expected Remove command"),
    }
}

#[test]
fn test_special_characters_in_args() {
    let cli = parse_args(&["pb", "tag", "/path with spaces/", "tag with spaces and @#$% symbols"]);
    match cli.command {
        Commands::Tag { path, message } => {
            assert_eq!(path, "/path with spaces/");
            assert_eq!(message, "tag with spaces and @#$% symbols");
        }
        _ => panic!("Expected Tag command"),
    }
}

#[test]
fn test_unicode_in_args() {
    let cli = parse_args(&["pb", "tag", "/路径/パス/경로", "标签 タグ 태그"]);
    match cli.command {
        Commands::Tag { path, message } => {
            assert_eq!(path, "/路径/パス/경로");
            assert_eq!(message, "标签 タグ 태그");
        }
        _ => panic!("Expected Tag command"),
    }
}

#[test]
fn test_long_arguments() {
    let long_path = "/".repeat(100);
    let long_tag = "a".repeat(100);
    
    let cli = parse_args(&["pb", "tag", &long_path, &long_tag]);
    match cli.command {
        Commands::Tag { path, message } => {
            assert_eq!(path, long_path);
            assert_eq!(message, long_tag);
        }
        _ => panic!("Expected Tag command"),
    }
}

#[test]
fn test_command_error_cases() {
    assert!(Cli::try_parse_from(&["pb", "tag"]).is_err());
    assert!(Cli::try_parse_from(&["pb", "search"]).is_err());
    assert!(Cli::try_parse_from(&["pb", "find"]).is_err());
} 