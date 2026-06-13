load("@aspect_bazel_lib//lib:write_source_files.bzl", "write_source_files")

util = struct(
    write_source_files = write_source_files,
)
