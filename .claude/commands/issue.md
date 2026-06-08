Create or update a GitHub issue.

## Conventions

**Title:** a short description, no trailing period.

**Label:** one of the following:

* `bug`: something is broken.
* `enhancement`: improvement to existing functionality.
* `idea`: new project or concept to explore.

**Body:** complete sentences, correct punctuation, Markdown allowed.

Bug:

* What is broken.
* Why it matters.
* Steps to reproduce.
* Expected vs. actual behavior.

Enhancement:

* What to improve.
* Why it matters.
* Current vs. desired behavior.

Idea:

* What the project or concept is.
* Why it matters.
* Rough thoughts on scope or direction.

## Steps

### 1. Determine the action

If an issue number is provided (e.g. `/issue 123`), update that issue.
Otherwise, create a new issue.

### 2. Gather context

Run in parallel:

- `gh issue list` to check for existing related issues.
- If updating, `gh issue view <number>` to read the current issue.

### 3. Draft the title and body

Follow the conventions above.

### 4. Create or update the issue

If creating:

```bash
gh issue create \
  --title "<title>" \
  --label "<label>" \
  --body "<body>"
```

If updating:

```bash
gh issue edit <number> \
  --title "<title>" \
  --label "<label>" \
  --body "<body>"
```

### 5. Return the issue URL
