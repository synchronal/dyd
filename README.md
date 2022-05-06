# DYD

[![CI](https://github.com/synchronal/dyd/actions/workflows/tests.yml/badge.svg)](https://github.com/synchronal/dyd/actions)
[![Hex pm](http://img.shields.io/crates/v/dyd.svg?style=flat)](https://crates.io/crates/dyd)
[![License](http://img.shields.io/github/license/synchronal/dyd.svg?style=flat)](https://github.com/synchronal/dyd/blob/main/LICENSE.md)

Daily diff.

This command line tool facilitates the viewing of git diffs across multiple projects, across
multiple days. See what you're doing across teams, and across all the git repos your teams
manage.


## Installation

```shell
cargo install dyd
```

Configure git with a GUI difftool:

```toml
[diff]
  tool = Kaleidoscope
  guitool = Kaleidoscope
[difftool]
  prompt = false
[difftool "Kaleidoscope"]
  cmd = ksdiff --partial-changeset --relative-path \"$MERGED\" -- \"$LOCAL\" \"$REMOTE\"
[mergetool "Kaleidoscope"]
  cmd = ksdiff --merge --output \"$MERGED\" --base \"$BASE\" -- \"$LOCAL\" --snapshot \"$REMOTE\" --snapshot
  trustExitCode = true
```


## Usage

Create a manifest file at `dyd.toml` with the following format:

```toml
since = "3 days ago"
#       "  ^^^^ days | weeks | months

## difftool = "my diff tool"

[remotes]

[remotes.dyd]
name = "DYD"
origin = "git@github.com:synchronal/dyd"

[remotes.tui]
name = "TUI"
origin = "git@github.com:fdehau/tui-rs"
```

Ensure that your shell is authorized with the origin. DYD will *not* route input to the SSH agent.

```shell
ssh-add ~/.ssh/id_ed25519
```

Open the diff tool:

```shell
dyd -m dyd.toml
dyd --manifest dyd.toml
```

Keymap:

```
h l <left> <right> <tab> - switch panes
j k <up> <down> - change current selection
d - open git gui difftool
q <esc> - quit
```


## Other difftools

IntelliJ IDEA:

```toml
[diff]
  guitool = intellij
[difftool "intellij"]
  cmd = /Applications/IntelliJ\\ IDEA\\ CE.app/Contents/MacOS/idea diff $(cd $(dirname "$LOCAL") && pwd)/$(basename "$LOCAL") $(cd $(dirname "$REMOTE") && pwd)/$(basename "$REMOTE")
```

Manifest:

```toml
difftool = "git difftool --dir-diff -g -y"
```

TODO:
- gitx
- kdiff3
- ???


## References

- https://github.com/orhun/rust-tui-template

