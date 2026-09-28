load("@rules_rust//rust:defs.bzl", "rust_proc_macro")

rust_proc_macro(
    name = "cas_cbor2",
    srcs = ["src/lib.rs"],
    visibility = ["//visibility:public"],
    deps = [
        "@crates//:proc-macro2",
        "@crates//:quote",
        "@crates//:syn",
    ],
)
