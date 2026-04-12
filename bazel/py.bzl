load(
    "@rules_python//python:defs.bzl",
    "py_binary",
    "py_library",
    "py_test",
)

py = struct(
    binary = py_binary,
    library = py_library,
    test = py_test,
)
