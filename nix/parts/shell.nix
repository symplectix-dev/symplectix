{ ... }:
{
  perSystem =
    { pkgs, config, ... }:
    let
      bazel = pkgs.writeShellScriptBin "bazel" ''
        exec ${pkgs.bazelisk}/bin/bazelisk "$@"
      '';

      # `rules_rust`'s helix setup prints a `.helix/languages.toml` snippet that
      # nests config under an extra `rust-analyzer.` level Helix doesn't strip,
      # so rust-analyzer silently ignores it; it also relies on `bazel` being on
      # PATH for the discover command. This wraps the generator and fixes both.
      rust-analyzer-setup-helix = pkgs.writeShellScriptBin "rust-analyzer-setup-helix" ''
        set -euo pipefail
        root=$(${pkgs.git}/bin/git rev-parse --show-toplevel)
        {
          cat <<'EOF'
# Append "{arg}" to the end of the discoverConfig command array below for
# per-package workspace discovery: rust-analyzer reruns discover scoped to
# the package of the currently opened file instead of the whole repo.
# Faster on large monorepos and needed if `bazel query //...` does not
# load cleanly here, but switching to a file in a different package
# triggers a reload, and dependents of the package are not indexed.
EOF
          ${bazel}/bin/bazel run @rules_rust//tools/rust_analyzer:setup -- helix \
            | ${pkgs.gnused}/bin/sed -E 's/\[language-server\.rust-analyzer\.config\.rust-analyzer\./[language-server.rust-analyzer.config./' \
            | ${pkgs.gnused}/bin/sed 's#discover_bazel_rust_project.exe"#discover_bazel_rust_project.exe", "--bazel", "${bazel}/bin/bazel"#'
        } > "$root/.helix/languages.toml"
        echo "wrote $root/.helix/languages.toml"
      '';

      venvHook = ''
        _root=$(${pkgs.git}/bin/git rev-parse --show-toplevel)
        if [[ ! -d "$_root/.venv" ]]; then
          ${bazel}/bin/bazel run //:create_venv
        fi
        if [[ -d "$_root/.venv" ]]; then
          source "$_root/.venv/bin/activate"
        fi
        unset _root
      '';
    in
    {
      # Shell settings for interactive use.
      devShells.default = pkgs.mkShellNoCC {
        # Hook packages are managed separately in hooks.nix and not included here.
        # To also expose hook packages in the shell, prepend:
        #   config.pre-commit.settings.enabledPackages ++
        packages = with pkgs; [
          bazel
          bazel-buildtools
          rust-analyzer-setup-helix
          ruff
          basedpyright
          github-cli
          jq
          protobuf
          ansible
        ];

        shellHook = ''
          ${config.pre-commit.shellHook}
          ${venvHook}
        '';
      };

      # Shell settings for CI.
      devShells.ci = pkgs.mkShellNoCC {
        packages = with pkgs; [
          bazel
          prek
        ];

        shellHook = ''
          ${config.pre-commit.shellHook}
          ${venvHook}
        '';
      };
    };
}
