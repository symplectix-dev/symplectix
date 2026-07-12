# syx

Short alias for the `symplectix` namespace.
Each entry under `syx/` is a directory-based Python package with its own `BUILD.bazel`.

## Build and test

```sh
bazel build //syx/...
bazel test //syx/...
```

## Rust Analyzer

Set up an editor to use rust-analyzer against the Bazel-built toolchain
(matches the pinned rustc, no separate Cargo project needed):

```sh
bazel run @rules_rust//tools/rust_analyzer:setup -- vscode
bazel run @rules_rust//tools/rust_analyzer:setup -- neovim
bazel run @rules_rust//tools/rust_analyzer:setup -- helix
bazel run @rules_rust//tools/rust_analyzer:setup -- print   # editor-agnostic JSON, e.g. for coc.nvim
```

Re-run after adding/removing crates or changing deps. See `-- --help` (and
each subcommand's own `--help`) for options.

## Rust Benchmarks

Some crates have `criterion` benchmarks, built as plain `rust.binary`
targets (Cargo's `harness = false` benches are just binaries with their own
`main` via `criterion_main!`):

Run one with:

```sh
bazel run //syx/bitcomp_test:rrr_encode_decode_bench -- --bench
```

By default, criterion writes its report under a `bazel run` runfiles
directory that is effectively throwaway. Set `CRITERION_HOME` to control
where the report goes:

```sh
CRITERION_HOME=/tmp/bench-out \
  bazel run //syx/bitcomp_test:rrr_encode_decode_bench -- --bench
```

Report ends up at `$CRITERION_HOME/report/index.html`.

## Rust Fuzzing

`rust.fuzz_binary` (`//bazel/private:rust_fuzz_binary.bzl`) builds a
libFuzzer/ASan binary from a `#![no_main]` crate using `libfuzzer_sys::fuzz_target!`.
See `//syx/fuzz_examples` for working examples. Run one with:

```sh
bazel run //syx/fuzz_examples:buffer_overflow
```

### Transitioned away from `hermetic_cc_toolchain`

`zig cc` (0.12.0, bundled by hermetic_cc_toolchain 4.1.0) errors out on some
linker flags rustc passes for sanitizer builds:

```
error: unsupported linker arg: .../librustc-nightly_rt.asan.a
```

Known upstream: [ziglang/zig#16813], still open.

`rust.fuzz_binary` works around it with a Starlark transition
(`//bazel/private:fuzz_transition.bzl`) applied to the underlying `rust_binary`.

`--config=fuzz` exists for optimization settings (e.g., LTO, codegen units)
that aren't required for correctness, only for less painfully slow
fuzzing runs -- toolchain selection does not depend on it.

Requires a working host clang/gcc in dev and CI environments.

[ziglang/zig#16813]: https://github.com/ziglang/zig/issues/16813
