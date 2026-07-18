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

rust.test(
    name = "ply_test",
    size = "small",
    crate = ":ply",
    data = ["//:.rustfmt.toml"],
    deps = [
        "//syx/testing",
        "@crates//:tokio",
    ],
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
