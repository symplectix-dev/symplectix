load("@rules_uv//uv:pip.bzl", "pip_compile")
load("@rules_uv//uv:venv.bzl", "create_venv")

exports_files([
    "MODULE.bazel",
    ".clippy.toml",
    ".rustfmt.toml",
])

pip_compile(
    name = "pip_compile",
    requirements_in = ":pyproject.toml",
    requirements_txt = ":requirements.txt",
)

create_venv(
    name = "create_venv",
    destination_folder = ".venv",
    requirements_txt = ":requirements.txt",
)
