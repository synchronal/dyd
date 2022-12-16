# Change log

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

