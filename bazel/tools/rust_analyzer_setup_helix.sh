#!/usr/bin/env bash
# Writes .helix/languages.toml for rust-analyzer.

set -euo pipefail

bazel=bazel
root=

while [ $# -gt 0 ]; do
  case $1 in
  --bazel | --root)
    if [ $# -lt 2 ]; then
      echo "$1 requires an argument" >&2
      exit 2
    fi
    case $1 in
    --bazel) bazel=$2 ;;
    --root) root=$2 ;;
    esac
    shift 2
    ;;
  *)
    echo "unexpected argument: $1" >&2
    exit 2
    ;;
  esac
done

if [ -z "$root" ]; then
  root=$(git rev-parse --show-toplevel)
fi

# `rules_rust`'s helix setup prints a snippet that nests config under
# an extra `rust-analyzer.` level Helix does not strip, so rust-analyzer
# silently ignores it. The snippet also assumes `bazel` is on PATH of
# whatever launched the editor, and omits any check command, leaving
# rust-analyzer on its `cargo check` default. This wraps the generator
# and patches those up.
#
# The "{arg}" the generator puts in the discover command keeps
# per-package workspace discovery on: rust-analyzer reruns discover
# scoped to the package of the file just opened, so switching to another
# package triggers a reload and dependents go unindexed.
config=$("$bazel" run @rules_rust//tools/rust_analyzer:setup -- helix | sed -E \
  -e 's/\[language-server\.rust-analyzer\.config\.rust-analyzer\./[language-server.rust-analyzer.config./' \
  -e "s#discover_bazel_rust_project.exe\"#discover_bazel_rust_project.exe\", \"--bazel\", \"$bazel\"#")

# The flycheck runnable in the discover output invokes flycheck.exe
# without --bazel, so its inner bazel resolves against the PATH of
# whatever launched the editor and fails there. Overriding check drives
# the same wrapper in its override mode with bazel pinned, where it
# derives the label from the saved file rather than from discover.
check=$(
  cat <<EOF
[language-server.rust-analyzer.config.check]
overrideCommand = ["$root/.helix/.rules_rust_analyzer/flycheck.exe", "--bazel", "$bazel", "--saved-file", "\$saved_file"]
EOF
)

mkdir -p "$root/.helix"
printf '%s\n\n%s\n' "$config" "$check" >"$root/.helix/languages.toml"
