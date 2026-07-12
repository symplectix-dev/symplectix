load("@rules_cc//cc:cc_binary.bzl", "cc_binary")
load("@rules_cc//cc:cc_library.bzl", "cc_library")

cc = struct(
    binary = cc_binary,
    library = cc_library,
)
