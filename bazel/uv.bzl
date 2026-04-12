load("@rules_uv//uv:pip.bzl", _pip_compile = "pip_compile")
load("@rules_uv//uv:venv.bzl", _create_venv = "create_venv")

visibility("//")

uv = struct(
    pip_compile = _pip_compile,
    create_venv = _create_venv,
)
