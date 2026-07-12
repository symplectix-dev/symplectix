visibility(["//bazel:__pkg__"])

def _fuzz_transition_impl(_settings, _attr):
    return {
        # Keeps zig out of the picture.
        #
        # Toolchain resolution for `@bazel_tools//tools/cpp:toolchain_type` normally walks
        # `register_toolchains()` in order and picks the first `toolchain()` whose
        # `target_compatible_with` is satisfied by the target platform; the zig toolchains
        # registered in MODULE.bazel win that race for the default host platform.
        # `--extra_toolchains` is consulted *before* the registered list, so pointing it
        # at rules_cc's autodetected host toolchain forces toolchain resolution to prefer
        # it here without touching global registration order.
        #
        # ":all" (not a specific target like ":cc-toolchain-k8") so constraint
        # matching picks the right toolchain for the current host.
        "//command_line_option:extra_toolchains": ["@local_config_cc_toolchains//:all"],
        # rustc's `-Zsanitizer=` flag, added unconditionally by
        # `rust_fuzz_binary` below, only works on a nightly compiler.
        "@rules_rust//rust/toolchain/channel": "nightly",
    }

fuzz_transition = transition(
    implementation = _fuzz_transition_impl,
    inputs = [],
    outputs = [
        "//command_line_option:extra_toolchains",
        "@rules_rust//rust/toolchain/channel",
    ],
)

def _fuzz_transition_wrapper_impl(ctx):
    # `actual` went through an outgoing-edge transition, so ctx.attr.actual
    # is a list (one element per resulting configuration -- always exactly
    # one here, since fuzz_transition doesn't split into multiple).
    actual = ctx.attr.actual[0]
    actual_default_info = actual[DefaultInfo]
    actual_executable = actual_default_info.files_to_run.executable

    # This rule must produce its own executable, so symlink to
    # the dependency's real binary.
    executable = ctx.actions.declare_file(ctx.label.name)
    ctx.actions.symlink(output = executable, target_file = actual_executable, is_executable = True)

    # Forward the dependency's runfiles (shared libs, data, etc.) plus the
    # new symlink itself, so running this target behaves like running the
    # real binary directly.
    runfiles = actual_default_info.default_runfiles.merge(ctx.runfiles([executable]))

    providers = [
        DefaultInfo(
            executable = executable,
            files = depset([executable]),
            runfiles = runfiles,
        ),
    ]

    # Forward output groups too (e.g. pyo3_type_stubs, which this repo's
    # .bazelrc always requests via --output_groups=+pyo3_type_stubs) so
    # building through the wrapper doesn't silently drop them.
    if OutputGroupInfo in actual:
        providers.append(actual[OutputGroupInfo])

    return providers

fuzz_transition_wrapper = rule(
    implementation = _fuzz_transition_wrapper_impl,
    attrs = {
        "actual": attr.label(
            cfg = fuzz_transition,
            mandatory = True,
            executable = True,
        ),
    },
    executable = True,
)
