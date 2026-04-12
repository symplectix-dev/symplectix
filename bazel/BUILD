load("@buildifier_prebuilt//:rules.bzl", "buildifier", "buildifier_test")

_exclude_patterns = [
    "./.git/*",
    "./.direnv/*",
]

_lint_warnings = [
    "-function-docstring",
    "-function-docstring-args",
    "-module-docstring",
]

buildifier(
    name = "buildifier",
    exclude_patterns = _exclude_patterns,
    lint_mode = "fix",
    lint_warnings = _lint_warnings,
)

buildifier_test(
    name = "buildifier_test",
    size = "small",
    timeout = "short",
    exclude_patterns = _exclude_patterns,
    lint_mode = "warn",
    lint_warnings = _lint_warnings,
    mode = "diff",
    no_sandbox = True,
    workspace = "//:MODULE.bazel",
)
