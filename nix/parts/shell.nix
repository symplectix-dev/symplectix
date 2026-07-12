{ ... }:
{
  perSystem =
    { pkgs, config, ... }:
    let
      bazel = pkgs.writeShellScriptBin "bazel" ''
        exec ${pkgs.bazelisk}/bin/bazelisk "$@"
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
