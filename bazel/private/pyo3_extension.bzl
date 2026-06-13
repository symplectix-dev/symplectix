load("@aspect_bazel_lib//lib:write_source_files.bzl", "write_source_files")
load("@rules_rust_pyo3//:defs.bzl", _pyo3_extension = "pyo3_extension")

visibility(["//bazel:__pkg__"])

_STUB_HEADER = (
    "printf " +
    "'# ruff: noqa\\n# This file is auto-generated. DO NOT EDIT MANUALLY.\\n'" +
    " | cat - $(SRCS) > $@"
)

def _pyo3_extension_impl(name, visibility, srcs, deps):
    # write_source_files references the checked-in .pyi as a label.
    # Without this declaration, Bazel cannot resolve that label
    # even though the file exists on disk.
    native.exports_files([name + ".pyi"])

    _pyo3_extension(
        name = name,
        srcs = srcs,
        deps = deps,
        stubs = True,
        visibility = visibility,
    )

    native.filegroup(
        name = name + "_stubs",
        srcs = [":" + name],
        output_group = "pyo3_type_stubs",
        visibility = ["//visibility:private"],
    )

    native.genrule(
        name = name + "_stubs_renamed",
        srcs = [":" + name + "_stubs"],
        outs = [name + "_renamed.pyi"],
        cmd = _STUB_HEADER,
        visibility = ["//visibility:private"],
    )

    write_source_files(
        name = name + "_stubs_update",
        files = {
            name + ".pyi": ":" + name + "_stubs_renamed",
        },
        # write_source_files checks file existence via native.glob(), which is
        # forbidden in symbolic macros. Disabling it avoids that restriction;
        # the golden test still catches a missing or stale file at test time.
        check_that_out_file_exists = False,
        visibility = visibility,
    )

pyo3_extension = macro(
    implementation = _pyo3_extension_impl,
    attrs = {
        "srcs": attr.label_list(allow_files = [".rs"]),
        "deps": attr.label_list(),
    },
)
