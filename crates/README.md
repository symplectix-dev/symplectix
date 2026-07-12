# crates

Rust crates built with Bazel.
See each `BUILD.bazel`; external deps are declared in `//bazel/bzlmod:crate.MODULE.bazel`.

## Build and test

```sh
bazel build //crates/...
bazel test //crates/...
```

## Benchmarks

Some crates have `criterion` benchmarks, built as plain `rust.binary`
targets (Cargo's `harness = false` benches are just binaries with their own
`main` via `criterion_main!`):

Run one with:

```sh
bazel run //crates/bitcomp_test:rrr_encode_decode_bench -- --bench
```

By default, criterion writes its report under a `bazel run` runfiles
directory that is effectively throwaway. Set `CRITERION_HOME` to control
where the report goes:

```sh
CRITERION_HOME=/tmp/bench-out \
  bazel run //crates/bitcomp_test:rrr_encode_decode_bench -- --bench
```

Report ends up at `$CRITERION_HOME/report/index.html`.

## Fuzzing

`rust.fuzz_binary` (`//bazel/internal:rust_fuzz_binary.bzl`) builds a
libFuzzer/ASan binary from a `#![no_main]` crate using `libfuzzer_sys::fuzz_target!`.
See `//fuzz_examples` for working examples. Run one with:

```sh
bazel run --config=fuzz //fuzz_examples:buffer_overflow
```

### Why `--config=fuzz` doesn't use the hermetic zig toolchain

`zig cc` (0.12.0, bundled by hermetic_cc_toolchain 4.1.0) errors out on some
linker flags rustc passes for sanitizer builds:

```
error: unsupported linker arg: .../librustc-nightly_rt.asan.a
```

This is a `zig cc` driver limitation (known upstream: [ziglang/zig#16813],
still open), not a bug in this repo's Bazel config.

Fixed instead by steering `--config=fuzz` to the host's autodetected cc
toolchain (via `rules_cc`'s `cc_configure_extension`, not registered by
default so it doesn't affect normal builds) with `--extra_toolchains`, which
outranks `register_toolchains` order:

- `MODULE.bazel`: `cc_configure = use_extension("@rules_cc//cc:extensions.bzl", "cc_configure_extension")`
  + `use_repo(cc_configure, "local_config_cc_toolchains")`
- `bazel/bazelrc/profile.bazelrc`: `build:fuzz --extra_toolchains=@local_config_cc_toolchains//:cc-toolchain-k8`

Verified with `bazel run --config=fuzz //fuzz_examples:buffer_overflow`: links
successfully and AddressSanitizer correctly reports the injected
heap-buffer-overflow. Requires a working host clang/gcc in dev and CI
environments -- sanitizer fuzzing is inherently a single-machine,
non-hermetic activity anyway, so this is a small trade-off.

[ziglang/zig#16813]: https://github.com/ziglang/zig/issues/16813
