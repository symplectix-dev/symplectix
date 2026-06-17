#!/usr/bin/env python3
"""Helper to stamp workspace status.

```console
bazel run @rules_python//python/bin:repl --@rules_python//python/bin:repl_dep=@//:workspace_status <<'EOF'
from workspace_status import _strip_credentials
print(_strip_credentials("https://user:token@github.com/owner/repo.git"))
print(_strip_credentials("https://github.com/owner/repo.git"))
print(_strip_credentials("git@github.com:owner/repo.git"))
EOF
```
"""  # noqa: E501

import datetime as dt
import os
import shutil
import subprocess
from typing import Literal, overload
from urllib.parse import urlparse, urlunparse

_GIT = shutil.which("git") or "git"
_NULL_SHA = "0000000000"


@overload
def _run(prog: str, *args: str, check: Literal[True] = ...) -> str: ...
@overload
def _run(prog: str, *args: str, check: Literal[False]) -> str | None: ...
def _run(prog: str, *args: str, check: bool = True) -> str | None:
    result = subprocess.run(  # noqa: S603
        [prog, *args],
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        check=check,
    )
    if not check and result.returncode != 0:
        return None
    return result.stdout.rstrip()


@overload
def _git(*args: str, check: Literal[True] = ...) -> str: ...
@overload
def _git(*args: str, check: Literal[False]) -> str | None: ...
def _git(*args: str, check: bool = True) -> str | None:
    return _run(_GIT, *args, check=check)


def _strip_credentials(url: str) -> str:
    parsed = urlparse(url)
    if parsed.username or parsed.password:
        netloc = parsed.hostname or ""
        if parsed.port:
            netloc += f":{parsed.port}"
        return urlunparse(parsed._replace(netloc=netloc))
    return url


class WorkspaceStatus:
    """Bazel workspace status for build stamping."""

    def __init__(
        self,
        now: dt.datetime,
        *,
        repo_url: str | None = None,
        branch: str = "unknown",
        commit: str = _NULL_SHA,
        clean: bool = False,
        rev_count: int = 0,
        run_number: int = 0,
    ) -> None:
        """Initialize workspace status."""
        (year, week, _) = now.isocalendar()
        self.yy = year - 2000
        self.week = week
        self.repo_url = repo_url
        self.commit_sha = commit
        self.branch = branch
        self.clean = clean
        self.rev_count = rev_count
        self.run_number = run_number

    def print(self) -> None:
        """Print key-value pairs for Bazel workspace status command."""
        count = self.run_number or self.rev_count
        version_tag = f"{self.yy}.{self.week}.{count}+r{self.commit_sha}"
        print(f"REPO_URL {self.repo_url or ''}")
        print(f"COMMIT_SHA {self.commit_sha}")
        print(f"GIT_BRANCH {self.branch}")
        print(f"GIT_STATUS {'Clean' if self.clean else 'Dirty'}")
        print(f"STABLE_VERSION_TAG {version_tag}")


def _collect(now: dt.datetime) -> WorkspaceStatus:
    if _git("rev-parse", "--git-dir", check=False) is None:
        return WorkspaceStatus(now)

    remote = _git("remote", "get-url", "origin", check=False)
    repo_url = _strip_credentials(remote) if remote else None
    # GITHUB_HEAD_REF is set on pull_request events (the PR's source branch).
    # GITHUB_REF_NAME is set on push events (the branch or tag name).
    # Both are unset outside GitHub Actions, so fall back to git.
    branch = (
        os.environ.get("GITHUB_HEAD_REF")
        or os.environ.get("GITHUB_REF_NAME")
        or _git("rev-parse", "--abbrev-ref", "HEAD")
    )
    commit = _git("rev-parse", "--short=10", "HEAD")
    clean = _git("diff-index", "--quiet", "HEAD", "--", check=False) is not None

    monday = now - dt.timedelta(days=now.weekday())
    since = monday.strftime("%Y-%m-%dT%H:%M:%S%z")
    rev_count = int(_git("rev-list", "--count", "HEAD", "--since", since))

    # Github Actions environment variables.
    # https://docs.github.com/en/actions/learn-github-actions/contexts#github-context
    #
    # * RUN_ID: A unique number for each workflow run within a repository. This number does not
    #   change if you re-run the workflow run.
    #
    # * RUN_NUMBER: A unique number for each run of a particular workflow in a repository. This
    #   number begins at 1 for the workflow's first run, and increments with each new run. This
    #   number does not change if you re-run the workflow run.
    #
    # * RUN_ATTEMPT: A unique number for each attempt of a particular workflow run in a
    #   repository. This number begins at 1 for the workflow run's first attempt, and
    #   increments with each re-run.
    run_number = int(os.environ.get("GITHUB_RUN_NUMBER", "0"))

    return WorkspaceStatus(
        now,
        repo_url=repo_url,
        branch=branch,
        commit=commit,
        clean=clean,
        rev_count=rev_count,
        run_number=run_number,
    )


if __name__ == "__main__":
    if bwd := os.getenv("BUILD_WORKING_DIRECTORY"):
        os.chdir(bwd)

    _collect(dt.datetime.now(tz=dt.UTC)).print()
