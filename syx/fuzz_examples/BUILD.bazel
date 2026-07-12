load("//bazel:rust.bzl", "rust")

rust.fuzz_binary(
    name = "should_not_fail",
    srcs = ["should_not_fail.rs"],
    tags = ["manual"],
    deps = [
        "@crates//:libfuzzer-sys",
    ],
)

rust.fuzz_binary(
    name = "buffer_overflow",
    srcs = ["buffer_overflow.rs"],
    tags = ["manual"],
    deps = [
        "@crates//:libfuzzer-sys",
    ],
)

rust.fuzz_binary(
    name = "memory_leaks",
    srcs = ["memory_leaks.rs"],
    tags = ["manual"],
    deps = [
        "@crates//:libfuzzer-sys",
    ],
)

rust.fuzz_binary(
    name = "use_after_scope",
    srcs = ["use_after_scope.rs"],
    env = {
        "ASAN_OPTIONS": ":".join([
            "detact_stack_use_after_return=1",
        ]),
    },
    tags = ["manual"],
    deps = [
        "@crates//:libfuzzer-sys",
    ],
)
