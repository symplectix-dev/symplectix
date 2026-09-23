load("@aspect_bazel_lib//lib:write_source_files.bzl", "write_source_files")
load("@bazel_skylib//rules:select_file.bzl", "select_file")

util = struct(
    select_file = select_file,
    write_source_files = write_source_files,
)
