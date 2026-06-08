# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## Instructions

### General

Write comments/documents in English.
Keep text short and brief.

Prefer ASCII. Avoid non-ASCII characters such as:
- em dashes (—)
- curly quotes ("“”")
- ellipses (…)

### Dev environment

The shell environment is managed by Nix. Enter it with:

```sh
nix develop ./nix
# or, if direnv is configured, just `cd` into the repository.
```

This activates all the toolchains you need.