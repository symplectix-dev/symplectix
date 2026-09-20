#!/bin/sh
# Stamps workspace status for Bazel builds.
#
# Bazel runs this outside the sandbox, so it gets
# whatever the host has on PATH.

set -eu

# `bazel run` starts in the runfiles tree rather than
# where it was invoked from.
if [ -n "${BUILD_WORKING_DIRECTORY:-}" ]; then
    cd "$BUILD_WORKING_DIRECTORY"
fi

version="0.0"
commit="0000000000"
branch="unknown"
status="dirty"

if git rev-parse --git-dir >/dev/null 2>&1; then
    # ISO year %g and week %V: the ISO year is what %V counts weeks within
    # and the two disagree at New Year. 2027-01-01 falls in ISO week 53 of
    # 2026, which %y would label 27.53, a week that does not exist.
    version=$(TZ=UTC git show -s --date=format-local:'%g.%V' --format=%cd HEAD)

    commit=$(git rev-parse --short=10 HEAD)

    # GITHUB_HEAD_REF is set on pull_request events (the PR's source
    # branch) and GITHUB_REF_NAME on push events. On a push GITHUB_HEAD_REF
    # is set but empty, so both need an empty check, not just an unset one.
    branch=${GITHUB_HEAD_REF:-${GITHUB_REF_NAME:-$(git rev-parse --abbrev-ref HEAD)}}

    # `git diff-index` only compares what is already tracked, so a new
    # file that has never been added reads as clean while being built
    # from. `status --porcelain` counts it, and still says nothing about
    # ignored paths, which are deliberately not part of the source.
    # `--no-optional-locks` keeps this from refreshing the index, since
    # Bazel may run it alongside whatever else is holding the repository.
    if [ -z "$(git --no-optional-locks status --porcelain 2>/dev/null)" ]; then
        status="clean"
    fi
fi

# Every key is a STABLE fact about the repository.
# Changing one rebuilds whatever embedded it;
# a volatile key would leave a stale value behind.
printf 'STABLE_VERSION %s\n'    "$version"
printf 'STABLE_GIT_COMMIT %s\n' "$commit"
printf 'STABLE_GIT_BRANCH %s\n' "$branch"
printf 'STABLE_GIT_STATUS %s\n' "$status"
