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

PRs are squash-merged, so the PR title and body become the commit message in `git log`.
Write them as if they are that commit message.

Title:

```
<area>: <description>
```

* `<area>` identifies the part of the codebase being changed.
  e.g., a Bazel package name, `ci`, `nix`, `chore`
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

Footnotes (optional, skip if the issue number is unknown):

* `Fixes #123` — bug fix; closes the issue on merge.
* `Closes #123` — closes the issue on merge.
* `Part of #123` — partial progress; does not close the issue.
* `Related to #123` — related but does not close the issue.

### Issues

Title:

```
<area>: <description>
```

* `<area>` identifies the part of the codebase the issue belongs to,
  same as pull requests.
* No trailing period.

Use labels to classify the nature of the issue:

* `bug` — something is broken.
* `enhancement` — improvement to existing functionality.
* `idea` — new project or concept to explore.

For body, write in complete sentences with correct punctuation.
Markdown is allowed.

Bug (`bug`):

* Describe the problem and why it matters.
* Steps to reproduce.
* Expected behavior and actual behavior.

Enhancement (`enhancement`):

* Describe the improvement and why it matters.
* Current behavior and desired behavior.

Idea (`idea`):

* What the new project or concept is.
* Why it is worth exploring.
* Any rough thoughts on scope or direction.
