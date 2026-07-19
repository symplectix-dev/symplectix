load("//bazel:rust.bzl", "rust")

rust.library(
    name = "ply",
    srcs = [
        "src/blob.rs",
        "src/function.rs",
        "src/lib.rs",
        "src/storable.rs",
        "src/store.rs",
    ],
    deps = [
        "//syx/cas",
        "@crates//:cbor2",
        "@crates//:fjall",
        "@crates//:serde",
        "@crates//:tokio",
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
        "tests/store_chunks_large_content.rs",
        "tests/store_fidelity.rs",
        "tests/store_is_deterministic.rs",
        "tests/store_missing.rs",
        "tests/store_persists.rs",
        "tests/store_reads_into_a_writer.rs",
        "tests/store_skips_compression_when_not_worthwhile.rs",
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
        "@crates//:rand",
        "@crates//:tokio",
    ],
)
