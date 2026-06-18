# Check action

Runs pre-build checks, build, and test. Supports two modes:

- `presubmit`: builds only targets affected by the change.
- `postsubmit`: unconditionally builds `//...`.

## presubmit

[bazel-diff] computes impacted targets by hashing the Bazel target graph at the
base commit and at HEAD, then diffing the two hash sets. If no targets are
affected, Build and Test are skipped entirely.

The impacted target list is printed in the "Compute impacted targets" step log,
so a suspiciously short or empty list is visible in Actions.

Base hashes are restored from a cache saved by the preceding postsubmit run.
The cache key is `bazel-diff-<os>-<base-sha>`, where `base-sha` equals
`head-sha` of the postsubmit run at that commit. On cache miss, base hashes
are generated via a git worktree at the base commit.

## postsubmit

Builds and tests `//...`. Also saves disk cache, repository cache and
bazel-diff hashes for use by the next presubmit run.

Bazel caches are saved only in postsubmit to prevent short-lived PR branches
from polluting caches shared across runs.

Head hashes are saved before Build and Test so the cache is populated even if
the build fails, ensuring the next presubmit always has base hashes to restore.

Postsubmit runs on every push to main and weekly on Monday.

[bazel-diff]: https://github.com/Tinder/bazel-diff
