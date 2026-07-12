load(
    "@bazel_skylib//rules:run_binary.bzl",
    _run_binary = "run_binary",
)

lib = struct(
    run_binary = _run_binary,
)
