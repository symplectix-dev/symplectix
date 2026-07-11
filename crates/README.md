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
