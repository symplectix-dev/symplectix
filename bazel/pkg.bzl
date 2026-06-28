load("@rules_pkg//pkg:install.bzl", "pkg_install")
load("@rules_pkg//pkg:mappings.bzl", "pkg_attributes", "pkg_filegroup", "pkg_files")

pkg = struct(
    attributes = pkg_attributes,
    files = pkg_files,
    filegroup = pkg_filegroup,
    install = pkg_install,
)
