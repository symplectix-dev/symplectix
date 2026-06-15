load("@aspect_bazel_lib//lib:write_source_files.bzl", "write_source_files")
load("@rules_rust_pyo3//:defs.bzl", _pyo3_extension = "pyo3_extension")

visibility(["//bazel:__pkg__"])

# Prepend a header to the generated stub so that
# ruff skips the file, "# ruff: noqa";
#
# per-file-ignores in pyproject.toml suppresses PGH004
# for *.pyi so that the "ruff: noqa" is accepted.
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

    # stubs = True embeds PYO3_INTROSPECTION_1_* symbols in the .so,
    # which pyo3-introspection reads to generate the .pyi at build time.
    _pyo3_extension(
        name = name,
        srcs = srcs,
        deps = deps,
        stubs = True,
        # Keep .symtab so pyo3_introspection can read PYO3_INTROSPECTION_* symbols.
        # rules_rust passes -Cstrip=debuginfo in opt mode, which Zig/LLD interprets
        # aggressively enough to also strip .symtab.
        # --keep-symbol and --export-dynamic-symbol are not in Zig's CC wrapper
        # allowlist, so -Cstrip=none is the only available workaround for now.
        rustc_flags = ["-Cstrip=none"],
        visibility = visibility,
    )

    native.filegroup(
        name = name + "_stubs",
        srcs = [":" + name],
        output_group = "pyo3_type_stubs",
        visibility = ["//visibility:private"],
    )

    # The generated stub has the same filename as the checked-in stub.
    # Bazel resolves both to same short path and one would shadow the other.
    # Renaming the generated file avoids this conflict so that
    # write_source_files can diff them correctly.
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
        # the golden test ({name}_stubs_update_test) still catches a missing
        # or stale file at test time.
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
