load("@rules_shell//shell:sh_binary.bzl", "sh_binary")
load("@rules_shell//shell:sh_library.bzl", "sh_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")

sh = struct(
    binary = sh_binary,
    library = sh_library,
    test = sh_test,
)
