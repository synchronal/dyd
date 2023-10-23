# Change log

## Unreleased

## 1.7.2

- Update of internal dependency `ratatui` with breaking changes.

## 1.7.1

- Switch `tui` dependency to `ratatui`.
- Fix possible future (nightly) regex incompatibility with optional lookahead in github difftool url.

## 1.7.0

- Difftool configuration defines an enum.
  - git - use default git gui difftool.
  - github - open the default browser to a github diff.
  - fallthrough - any unrecognized string is treated as the difftool command.
- Fix opening of difftool with branch specified.

## 1.6.1

- View logs from branches via `origin/{branch}` rather than switching branches.

## 1.6.0

- Allow branch to be specified for a repo.

## 1.5.4

- Add dyd version to help box.
- Change styling of Help section.

## 1.5.3

- Prune git repositories when pulling.
- Use Rust 1.70.0.

## 1.5.2

- Sort repos by _parsed_ datetime of most recent log.
- Fix handling of default subcommand with default args.

### 1.5.1

- Avoid panic when logs can't be parsed, ie when specifying a repo that
  does not actually exist.

### 1.5.0

- Show times in local timezone.

### 1.4.1

- Fix argument handling when running with implicit `diff` subcommand.

### 1.4.0

- When not specifying a subcommand, diff params are parsed.
- Manifest files can be specified via `DYD_MANIFEST_PATH`.

### 1.3.0

- Add `clean` subcommand.
- Add descriptions to subcommands in usage.

### 1.2.1

- Sort repos based on unix datetime, rather than on chrono::DateTime struct.

### 1.2.0

- Include `DYD_PWD` in env when opening difftool.
- Updates the README to better reflect recent changes.


### 1.1.1

Handle log parsing for repos with no commits.


### 1.1.0

Adds `r` binding to refresh all repos.


### 1.0.0

Adds subcommands (`diff`, `init`) with the default subcommand being `diff`. When not
specifying a subcommand, the default manifest path of `./dyd.toml` is used. In order
to specify a specific manifest file, use `dyd diff -m <path/to/file.toml>`.


### 0.5.0

Last non-1.0 release.

