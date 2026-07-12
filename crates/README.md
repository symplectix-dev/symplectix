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
bazel run //fuzz_examples:buffer_overflow
```

No `--config=fuzz` needed -- see below.

### Toolchain: transitioned away from zig cc, not via `--config`

`zig cc` (0.12.0, bundled by hermetic_cc_toolchain 4.1.0) errors out on some
linker flags rustc passes for sanitizer builds:

```
error: unsupported linker arg: .../librustc-nightly_rt.asan.a
```

This is a `zig cc` driver limitation (known upstream: [ziglang/zig#16813],
still open), not a bug in this repo's Bazel config.

`rust.fuzz_binary` works around it with a Starlark transition
(`//bazel/internal:fuzz_transition.bzl`) applied to the underlying
`rust_binary`, rather than requiring callers to pass `--config=fuzz`:

- `extra_toolchains` is transitioned to the host's autodetected cc toolchain
  (`rules_cc`'s `cc_configure_extension`, aliased in `MODULE.bazel` as
  `local_config_cc_toolchains`; not registered by default, so it doesn't
  affect normal builds).
- the rust toolchain channel is transitioned to `nightly`, since
  `-Zsanitizer=...` requires it.

The transitioned `rust_binary` can't be exposed directly (an outgoing-edge
transition needs a wrapping rule), so `rust.fuzz_binary` generates
`<name>_fuzz_target_impl` (the real `rust_binary`) plus `<name>_fuzz_target`
(a thin forwarding rule -- `fuzz_transition_wrapper` -- that applies the
transition and symlinks through the executable and runfiles).

`--config=fuzz` still exists for optimization settings (opt, LTO, codegen
units) that aren't required for correctness, only for less painfully slow
fuzzing runs -- toolchain selection no longer depends on it.

Verified with `bazel run //fuzz_examples:buffer_overflow` (no config flags):
links successfully and AddressSanitizer correctly reports the injected
heap-buffer-overflow. Requires a working host clang/gcc in dev and CI
environments -- sanitizer fuzzing is inherently a single-machine,
non-hermetic activity anyway, so this is a small trade-off.

[ziglang/zig#16813]: https://github.com/ziglang/zig/issues/16813
