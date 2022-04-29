# DYD

Daily diff.

This command line tool facilitates the viewing of git diffs across multiple projects, across
multiple days. See what you're doing across teams, and across all the git repos your teams
manage.


## Installation

```shell
cargo install dyd
```

Configure git with a GUI difftool:

```
# Kaleidoscope is a paid tool, with a CLI installable with `brew install --cask ksdiff`
difftool.Kaleidoscope.cmd=ksdiff --partial-changeset --relative-path "$MERGED" -- "$LOCAL" "$REMOTE"
diff.guitool=Kaleidoscope
```

- TODO: gitx
- TODO: kdiff3


## Usage

Create a manifest file at `dyd.toml` with the following format:

```toml
since = "3 days ago"
#       "  ^^^^ days | weeks | months

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


## References

- https://github.com/orhun/rust-tui-template

