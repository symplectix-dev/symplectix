load(
    "@rules_python//python:defs.bzl",
    "py_binary",
    "py_library",
    "py_test",
)
load("@symplectix_pypi//:requirements.bzl", "requirement")
load("//bazel/private:pyo3_extension.bzl", "pyo3_extension")

py = struct(
    binary = py_binary,
    extension = pyo3_extension,
    library = py_library,
    requirement = requirement,
    test = py_test,
)
