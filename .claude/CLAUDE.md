# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## Instructions

### General

Write comments/documents in English, in complete sentences with correct punctuation.
Keep text short and brief.

### Dev environment

The shell environment is managed by Nix. Enter it with:

```sh
nix develop ./nix
# or, if direnv is configured, just `cd` into the repository.
```

This activates all the toolchains you need.

### Pull requests

PRs are squash-merged, so the PR title and body become the commit message in `git log`. Write them as if they are that commit message.

Title:

```
<type>: <description>
```

* `<type>` consists of lowercase letters and classifies the PR.
  e.g., Bazel package name, `ci`, `bazel`, `chore`
* Uses the verb tense + phrase that completes the blank in:
  "This change modifies the project to ___________"
* Lowercase the verb.
* No trailing period.
* Keep the title short.

For example, `nix: add terraform to manage cloud resources`.

Body:

* Explain what and why, not how.
* Write in complete sentences with correct punctuation.
* No HTML, Markdown, or any other markup language.
* Bullet points are okay. Typically a hyphen or asterisk is
  used for the bullet, followed by a single space.
  Use a hanging indent.
* If the PR is related to a GitHub issue, include a reference
  at the end of the body. Skip this if the issue is unknown.
  - `Fixes #123` — bug fix; closes the issue on merge.
  - `Closes #123` — closes the issue on merge.
  - `Resolves #123` — closes the issue on merge.
  - `Part of #123` — partial progress; does not close the issue.
  - `Related to #123` — related but does not close the issue.
