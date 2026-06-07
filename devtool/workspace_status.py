#!/usr/bin/env python
"""Helper to stamp workspace."""

import datetime as dt
import os
import subprocess

from pydantic import BaseModel, Field


def git_rev_parse() -> str:
    """Return the working tree revision."""
    # A revision refers to the id you can use as a parameter
    # to reference an object in git (usually a commit).
    status = subprocess.run(
        ["git", "rev-parse", "--short=10", "HEAD"],  # noqa: S607
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        check=True,
    )
    return status.stdout.rstrip()


def git_rev_count(since: dt.datetime | None) -> int:
    """Return the number of revisions since the given date.

    If since is not specified, return the number of revisions since
    beginning of the history.

    Note:
    When you git clone, you need to retrieve enough history so that
    rev count returns an expected value.
    """
    args = ["git", "rev-list", "--count", "HEAD"]
    if since is not None:
        args.extend(
            [
                "--since",
                # not documented but git seems to support timezone in ISO 8601.
                # https://github.com/git/git/blob/master/Documentation/date-formats.adoc
                since.strftime("%Y-%m-%dT%H:%M:%S%z"),
            ]
        )
    status = subprocess.run(  # noqa: S603
        args,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        check=True,
    )
    return int(status.stdout.rstrip())


class WorkspaceStatus(BaseModel):
    """Bazel workspace status used for build stamping."""

    year: int = Field(description="")
    week: int = Field(description="")
    rev_count: int = Field(description="")
    rev_parse: str = Field(description="")


def workspace_status() -> WorkspaceStatus:
    """Return the version string."""
    now = dt.datetime.now(tz=dt.UTC)
    (year, week, _wday) = now.isocalendar()
    monday = now - dt.timedelta(days=now.weekday())
    rev_count = git_rev_count(monday)
    rev_parse = git_rev_parse()
    return WorkspaceStatus(
        year=year - 2000,
        week=week,
        rev_count=rev_count,
        rev_parse=rev_parse,
    )


if __name__ == "__main__":
    if bwd := os.getenv("BUILD_WORKING_DIRECTORY"):
        os.chdir(bwd)
    print(workspace_status().model_dump_json())
