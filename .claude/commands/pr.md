Create or update the pull request for the current branch.

## Conventions

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

* `Fixes #123`: bug fix; closes the issue on merge.
* `Closes #123`: closes the issue on merge.
* `Part of #123`: partial progress; does not close the issue.
* `Related to #123`: related but does not close the issue.

## Steps

### 1. Determine the base branch

Use the branch specified in the argument if provided (e.g. `/pr main`).
Otherwise default to `main`.

### 2. Run these in parallel

- `git log <base branch>..HEAD`
- `git diff <base branch>...HEAD`
- `gh pr view --json url,title,body` to check if a PR already exists for this branch.

### 3. Draft the title and body

Follow the conventions above.

### 4. Push the branch

```bash
git push -u origin HEAD
```

### 5. Create or update the PR

If no PR exists:

```bash
gh pr create \
  --draft \
  --base <base branch> \
  --title "<title>" \
  --body "<body>"
```

If a PR already exists:

```bash
gh pr edit \
  --title "<title>" \
  --body "<body>"
```

### 6. Return the PR URL
