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
          rust-toolchain
          rust-analyzer
          ruff
          basedpyright
          github-cli
          jq
          protobuf
          terramate
          ansible
        ];

        env = {
          RUST_SRC_PATH = "${pkgs.rust-toolchain}/lib/rustlib/src/rust/library";
        };

        shellHook = ''
          ${config.pre-commit.shellHook}
          ${venvHook}
        '';
      };

      # Shell settings for CI.
      devShells.ci = pkgs.mkShellNoCC {
        packages = with pkgs; [
          bazel
          rust-toolchain # can be removed once all crates/ are built by Bazel
          prek
        ];

        shellHook = ''
          ${config.pre-commit.shellHook}
          ${venvHook}
        '';
      };
    };
}
