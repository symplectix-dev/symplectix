load("//bazel:py.bzl", "py")
load("//bazel:uv.bzl", "uv")

py.binary(
    name = "workspace_status",
    srcs = ["workspace_status.py"],
)

exports_files([
    "MODULE.bazel",
    ".clippy.toml",
    ".rustfmt.toml",
])

uv.pip_compile(
    name = "pip_compile",
    requirements_in = ":pyproject.toml",
    requirements_txt = ":requirements.txt",
)

uv.create_venv(
    name = "create_venv",
    destination_folder = ".venv",
    requirements_txt = ":requirements.txt",
)
