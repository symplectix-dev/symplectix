load(
    "@rules_shell//shell:defs.bzl",
    "sh_binary",
    "sh_library",
    "sh_test",
)

sh = struct(
    binary = sh_binary,
    library = sh_library,
    test = sh_test,
)
