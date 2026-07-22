load("//bazel:rust.bzl", "rust")

rust.proc_macro(
    name = "cas_cbor2",
    srcs = ["src/lib.rs"],
    visibility = ["//visibility:public"],
    deps = [
        "@crates//:proc-macro2",
        "@crates//:quote",
        "@crates//:syn",
    ],
)
