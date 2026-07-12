load("@rules_python//python:defs.bzl", "py_binary")
load("@rules_rust//rust:defs.bzl", "rust_binary")

visibility("//bazel")

def _rust_fuzz_binary_impl(
        name,
        sanitizer,
        cfg_fuzzing,
        link_dead_code,
        trace_compares,
        trace_divs,
        trace_geps,
        disable_branch_folding,
        coverage,
        coverage_stack_depth,
        **kwargs):
    # https://github.com/rust-fuzz/cargo-fuzz
    # https://github.com/rust-fuzz/libfuzzer

    rustc_flags = []
    tags = ["fuzzing"]

    if cfg_fuzzing:
        rustc_flags.append("--cfg=fuzzing")

    if link_dead_code:
        rustc_flags.append("-Clink-dead-code")

    rustc_flags.extend([
        "-Cpasses=sancov-module",
        "-Cllvm-args=-sanitizer-coverage-level=4",
        "-Cllvm-args=-sanitizer-coverage-inline-8bit-counters",
        "-Cllvm-args=-sanitizer-coverage-pc-table",
    ])

    if trace_compares:
        rustc_flags.append("-Cllvm-args=-sanitizer-coverage-trace-compares")

    if trace_divs:
        rustc_flags.append("-Cllvm-args=-sanitizer-coverage-trace-divs")

    if trace_geps:
        rustc_flags.append("-Cllvm-args=-sanitizer-coverage-trace-geps")

    if disable_branch_folding:
        rustc_flags.append("-Cllvm-args=-simplifycfg-branch-fold-threshold=0")

    if coverage:
        rustc_flags.append("-Cinstrument-coverage")

    if coverage_stack_depth:
        rustc_flags.append("-Cllvm-args=-sanitizer-coverage-stack-depth")

    if sanitizer == "memory":
        rustc_flags.extend([
            "-Zsanitizer=memory",
            "-Zsanitizer-memory-track-origins",
        ])
    else:
        rustc_flags.append("-Zsanitizer={}".format(sanitizer))

    _rustc_flags = kwargs.pop("rustc_flags")
    if _rustc_flags == None:
        _rustc_flags = []

    _tags = kwargs.pop("tags")
    if _tags == None:
        _tags = []

    # if you change the suffix '_fuzz_target', you must update fuzztest.py too.
    rust_binary(
        name = "{}_fuzz_target".format(name),
        rustc_flags = _rustc_flags + rustc_flags,
        tags = _tags + tags,
        **kwargs
    )
    py_binary(
        name = name,
        srcs = ["//bazel/tools:fuzztest.py"],
        main = "//bazel/tools:fuzztest.py",
        args = [
            "$(rootpath :{}_fuzz_target)".format(name),
        ],
        data = [":{}_fuzz_target".format(name)],
        tags = _tags + tags,
    )

_rust_fuzz_binary = macro(
    inherit_attrs = rust_binary,
    implementation = _rust_fuzz_binary_impl,
    attrs = {
        "sanitizer": attr.string(
            configurable = False,
            doc = """
            Use a specific sanitizer, 'address' by default.
              https://doc.rust-lang.org/unstable-book/compiler-flags/sanitizer.html
            """,
            default = "address",
        ),
        "cfg_fuzzing": attr.bool(
            configurable = False,
            default = True,
        ),
        "link_dead_code": attr.bool(
            configurable = False,
            default = False,
        ),
        "coverage": attr.bool(
            configurable = False,
            doc = "Instrument program code with source-based code coverage information.",
            default = True,
        ),
        "trace_compares": attr.bool(
            configurable = False,
            default = True,
        ),
        "trace_divs": attr.bool(
            configurable = False,
            doc = """
            Enables `sanitizer-coverage-trace-divs` LLVM instrumentation.
              When set to True, the compiler will instrument integer division instructions
              to capture the right argument of division.
            """,
            default = True,
        ),
        "trace_geps": attr.bool(
            configurable = False,
            doc = """
            trace_geps: Enables `sanitizer-coverage-trace-geps` LLVM instrumentation
              When set to True, instruments GetElementPtr (GEP) instructions to track
              pointer arithmetic operations to capture array indices.
            """,
            default = True,
        ),
        "disable_branch_folding": attr.bool(
            configurable = False,
            doc = """
            disable_branch_folding: Disable transformation of if-statements into `cmov` instructions.
              When set to True, we get no coverage feedback for that branch. This is done by
              setting the `-simplifycfg-branch-fold-threshold=0` LLVM arg.
            """,
            default = True,
        ),
        "coverage_stack_depth": attr.bool(
            default = False,
        ),
    },
)

# Wraps _rust_fuzz_target to set default values to selectable expression.
def rust_fuzz_binary(**kwargs):
    env = kwargs.pop("env", {})

    asan_options = env.get("ASAN_OPTIONS")
    if asan_options == None:
        asan_options = "detect_odr_violation=0"
    else:
        asan_options += ":detect_odr_violation=0"

    tsan_options = env.get("TSAN_OPTIONS")
    if tsan_options == None:
        tsan_options = "report_signal_unsafe=0"
    else:
        tsan_options += ":report_signal_unsafe=0"

    env.update({
        "ASAN_OPTIONS": asan_options,
        "TSAN_OPTIONS": tsan_options,
    })

    _rust_fuzz_binary(
        env = env,
        coverage_stack_depth = select({
            "@platforms//os:linux": True,
            "//conditions:default": False,
        }),
        **kwargs
    )
