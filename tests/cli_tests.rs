use pathbrain::cli::{Cli, Commands};
use clap::Parser;

fn parse_args(args: &[&str]) -> Cli {
    Cli::try_parse_from(args).unwrap()
}

#[test]
fn test_tag_command() {
    let cli = parse_args(&["pb", "tag", "tag1", "tag2", "-p", "/test/path"]);
    match cli.command {
        Commands::Tag { path, tags } => {
            assert_eq!(path, Some("/test/path".to_string()));
            assert_eq!(tags, vec!["tag1", "tag2"]);
        }
        _ => panic!("Expected Tag command"),
    }

    let cli = parse_args(&["pb", "tag", "tag1", "tag2"]);
    match cli.command {
        Commands::Tag { path, tags } => {
            assert!(path.is_none());
            assert_eq!(tags, vec!["tag1", "tag2"]);
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
    let cli = parse_args(&["pb", "remove", "tag1", "tag2", "-p", "/test/path"]);
    match cli.command {
        Commands::Remove { path, tags, fuzzy } => {
            assert_eq!(path, Some("/test/path".to_string()));
            assert_eq!(tags, vec!["tag1", "tag2"]);
            assert!(!fuzzy);
        }
        _ => panic!("Expected Remove command"),
    }

    let cli = parse_args(&["pb", "remove"]);
    match cli.command {
        Commands::Remove { path, tags, fuzzy } => {
            assert!(path.is_none());
            assert!(tags.is_empty());
            assert!(!fuzzy);
        }
        _ => panic!("Expected Remove command"),
    }

    let cli = parse_args(&["pb", "untag", "tag1", "--path", "/test/path"]);
    match cli.command {
        Commands::Remove { path, tags, fuzzy } => {
            assert_eq!(path, Some("/test/path".to_string()));
            assert_eq!(tags, vec!["tag1"]);
            assert!(!fuzzy);
        }
        _ => panic!("Expected Remove command"),
    }
}

#[test]
fn test_special_characters_in_args() {
    let cli = parse_args(&["pb", "tag", "tag with spaces", "special@tag", "-p", "/path with spaces/"]);
    match cli.command {
        Commands::Tag { path, tags } => {
            assert_eq!(path, Some("/path with spaces/".to_string()));
            assert_eq!(tags, vec!["tag with spaces", "special@tag"]);
        }
        _ => panic!("Expected Tag command"),
    }
}

#[test]
fn test_unicode_in_args() {
    let cli = parse_args(&["pb", "tag", "标签", "タグ", "태그", "--path", "/路径/パス/경로"]);
    match cli.command {
        Commands::Tag { path, tags } => {
            assert_eq!(path, Some("/路径/パス/경로".to_string()));
            assert_eq!(tags, vec!["标签", "タグ", "태그"]);
        }
        _ => panic!("Expected Tag command"),
    }
}

#[test]
fn test_long_arguments() {
    let long_path = "/".repeat(100);
    let long_tag = "a".repeat(100);
    
    let cli = parse_args(&["pb", "tag", &long_tag, "-p", &long_path]);
    match cli.command {
        Commands::Tag { path, tags } => {
            assert_eq!(path, Some(long_path));
            assert_eq!(tags, vec![long_tag]);
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