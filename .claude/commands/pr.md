Create or update the pull request for the current branch following the conventions in CLAUDE.md.

## Steps

### 1. Determine the base branch

Use the branch specified in the argument if provided (e.g. `/pr main`).
Otherwise default to `main`.

### 2. Run these in parallel

- `git log <base branch>..HEAD`
- `git diff <base branch>...HEAD`
- `gh pr view --json url,title,body` to check if a PR already exists for this branch.

### 3. Draft the title and body

Follow the Pull requests section of CLAUDE.md.

### 4. Create or update the PR

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

### 5. Return the PR URL
