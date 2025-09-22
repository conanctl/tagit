# TagIt

A zero-dependency file tagging system built for terminal users. Tag any file or directory, then use fuzzy finder to jump anywhere in seconds.

![TagIt Demo](demos/github-demo-tiny.gif)


## Install

```bash
# From source (recommended)
git clone https://github.com/conanctl/tagit.git && cd tagit
cargo build --release && sudo cp target/release/tag /usr/local/bin/

# One-liner
curl -sSf https://raw.githubusercontent.com/conanctl/tagit/main/setup.sh | sh
```

## Quick Start

```bash
# Tag some locations
tag add ~/dotfiles "config"
tag add ~/Code/project "work"
tag add /etc/nginx "server config"

# List tags
tag ls
tag ls config

# Jump to locations (requires shell integration below)
tag jump
tag jump work

# Remove old unused tags
tag rm
tag rm config
```

### Shell Integration

Add to your `.bashrc` or `.zshrc`:

```bash
function tag() {
  if [ "$1" = "jump" ]; then
    local output="$(TAGIT_SHELL_INTEGRATION=1 tag "$@")"
    [[ -n "$output" && "$output" != ":" ]] && eval "$output"
  else
    command tag "$@"
  fi
}
```

## Usage

```bash
tag add <path> <tag>     # Tag a file/directory  
tag ls [pattern]         # List/search tags
tag jump [pattern]       # Interactive navigation
tag rm [pattern]         # Remove tags
```

## Philosophy

TagIt follows the Unix philosophy: small, focused, composable tools. It stores data locally, works offline, and integrates with your shell without trying to replace it.

Perfect for Vim users, system administrators, and anyone who lives in the terminal.

## License

MIT