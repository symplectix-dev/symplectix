load("@rules_rust//cargo:defs.bzl", "cargo_build_script")
load(
    "@rules_rust//rust:defs.bzl",
    "rust_binary",
    "rust_doc_test",
    "rust_library",
    "rust_library_group",
    "rust_proc_macro",
    "rust_test",
    "rust_test_suite",
)
load(
    "@rules_rust_prost//:defs.bzl",
    "rust_prost_library",
    "rust_prost_toolchain",
)
load(
    "//bazel/private:rust_fuzz_binary.bzl",
    "rust_fuzz_binary",
)

# TODO: benchmark
# https://github.com/criterion-rs/criterion.rs/blob/master/src/lib.rs

rust = struct(
    binary = rust_binary,
    build_script = cargo_build_script,
    library = rust_library,
    library_group = rust_library_group,
    doc_test = rust_doc_test,
    proc_macro = rust_proc_macro,
    prost_library = rust_prost_library,
    prost_toolchain = rust_prost_toolchain,
    test = rust_test,
    test_suite = rust_test_suite,
    fuzz_binary = rust_fuzz_binary,
)
