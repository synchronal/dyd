# DYD

[![CI](https://github.com/synchronal/dyd/actions/workflows/tests.yml/badge.svg)](https://github.com/synchronal/dyd/actions)
[![Hex pm](http://img.shields.io/crates/v/dyd.svg?style=flat)](https://crates.io/crates/dyd)
[![License](http://img.shields.io/github/license/synchronal/dyd.svg?style=flat)](https://github.com/synchronal/dyd/blob/main/LICENSE.md)

Daily diff.

This command line tool facilitates the viewing of git diffs across multiple projects, across
multiple days. See what you're doing across teams, and across all the git repos your teams
manage.


## Installation

via homebrew:

```shell
brew tap synchronal/tap
brew install dyd
```

via cargo:

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

- `since` - Must be in the format `<N>` `<units>` `ago`. Defaults to `1 week ago`.
- `difftool` - Optional command to run in order to open a diff. Defaults to `git difftool -g -y ${DIFF}`.
  - Variables that will be interpolated into the command, and also made available in the difftool
    ENV. **IMPORTANT**: variables will **only** be replaced if they are in the format `${VAR}`,
    including braces.
    - `DIFF` - in the format `@{u}..HEAD`.
    - `DYD_PWD` - the working directory that `dyd` was run from.
    - `ORIGIN` - the origin used to check out the repository, ie `git@github.com:<org>/<repo>(.git)?`
    - `REF_FROM` - the sha of the earlier commit of the diff.
    - `REF_TO` - the sha of the more recent commit of the diff. `HEAD`.
- `remotes` - a list of remote repositories to clone and pull.

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

### IntelliJ IDEA

(You can [download IntelliJ IDEA Community Edition](https://www.jetbrains.com/idea/download/#section=mac) 
for free, which has a pretty good graphical difftool.)

*Add to your `~/.gitconfig` file:*

```toml
[diff]
	tool = intellij
	guitool = intellij
[difftool "intellij"]
	cmd = open -nWa 'IntelliJ IDEA CE.app' --args diff $(realpath "$LOCAL") $(realpath "$REMOTE")
[difftool]
	prompt=false
```

*Add to your `dyd.toml` manifest file:*

```toml
difftool = "git difftool --dir-diff --tool=intellij -y ${DIFF}"
```

### Others
- gitx
- kdiff3
- ???


## References

- https://github.com/orhun/rust-tui-template

