load("@aspect_bazel_lib//lib:copy_file.bzl", "copy_file")
load("@aspect_bazel_lib//lib:write_source_files.bzl", "write_source_files")
load("@bazel_skylib//rules:select_file.bzl", "select_file")

util = struct(
    copy_file = copy_file,
    select_file = select_file,
    write_source_files = write_source_files,
)
