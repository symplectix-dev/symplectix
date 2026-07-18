load("//bazel:rust.bzl", "rust")

rust.library(
    name = "ply",
    srcs = [
        "src/blob.rs",
        "src/function.rs",
        "src/lib.rs",
        "src/storable.rs",
    ],
    deps = [
        "//syx/cas",
        "@crates//:cbor2",
        "@crates//:serde",
    ],
)

rust.test_suite(
    name = "ply_test_suite",
    size = "small",
    srcs = [
        "tests/command_digest_is_deterministic.rs",
        "tests/command_digest_is_sensitive.rs",
        "tests/function_digest_is_deterministic.rs",
        "tests/function_digest_is_sensitive.rs",
        "tests/function_resolves_from_store.rs",
        "tests/tree_digest_is_deterministic.rs",
        "tests/tree_digest_is_sensitive.rs",
        "tests/tree_entries.rs",
        "tests/tree_interns.rs",
    ],
    data = ["//:.rustfmt.toml"],
    shared_srcs = ["tests/common/mod.rs"],
    deps = [
        ":ply",
        "//syx/cas",
        "//syx/testing",
        "@crates//:tokio",
    ],
)
