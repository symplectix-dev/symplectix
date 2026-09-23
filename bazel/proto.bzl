load("@protobuf//bazel:proto_library.bzl", "proto_library")
load(
    "@rules_proto_grpc//:defs.bzl",
    "ProtoPluginInfo",
    "proto_compile_attrs",
    "proto_compile_impl",
    "proto_compile_toolchains",
    "proto_plugin",
)

# rules_proto_grpc expects a rule per plugin, with the plugin fixed in
# `_plugins`. Leaving that empty and taking the plugin at the call site
# through `extra_plugins` instead keeps which plugin runs visible where
# it is used, and means a plugin needs no rule of its own.
_proto_compile = rule(
    implementation = proto_compile_impl,
    attrs = dict(
        proto_compile_attrs,
        _plugins = attr.label_list(
            providers = [ProtoPluginInfo],
            default = [],
            cfg = "exec",
        ),
    ),
    toolchains = proto_compile_toolchains,
)

proto = struct(
    compile = _proto_compile,
    library = proto_library,
    plugin = proto_plugin,
)
