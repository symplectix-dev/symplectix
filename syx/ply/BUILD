load("//bazel:rust.bzl", "rust")

rust.library(
    name = "ply",
    srcs = [
        "src/action.rs",
        "src/blob.rs",
        "src/hash.rs",
        "src/lib.rs",
    ],
    deps = [
        "@crates//:cbor2",
        "@crates//:serde",
        "@crates//:serde_bytes",
        "@crates//:sha2",
    ],
)

rust.test(
    name = "ply_test",
    size = "small",
    crate = ":ply",
    data = ["//:.rustfmt.toml"],
    deps = ["//syx/testing"],
)

# rust.test_suite(
#     name = "ply_test_suite",
#     size = "small",
#     srcs = [
#         "tests/simple.rs",
#         "tests/thread.rs",
#     ],
#     shared_srcs = ["tests/helper/mod.rs"],
#     deps = [":plying"],
# )
