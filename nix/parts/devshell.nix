{ ... }:
{
  perSystem =
    { pkgs, config, ... }:
    let
      python = pkgs.python314;
      bazel = pkgs.writeShellScriptBin "bazel" ''
        exec ${pkgs.bazelisk}/bin/bazelisk $@
      '';
    in
    {
      devShells.default = pkgs.mkShellNoCC {
        # Tools for interactive use in the shell.
        # Hook packages are managed separately in githook.nix and not included here.
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
        ];

        env = {
          RUST_SRC_PATH = "${pkgs.rust-toolchain}/lib/rustlib/src/rust/library";
        };

        shellHook = config.pre-commit.shellHook + ''
          # Anchor to the repo root so paths are correct even when entering the shell
          # from a different working directory (e.g., `nix develop path/to/symplectix`).
          _root=$(git rev-parse --show-toplevel)

          # Guard the activate so a failed bazel run doesn't break the shell.
          if [[ ! -d "$_root/.venv" ]]; then
            bazel run //:create_venv
          fi
          if [[ -d "$_root/.venv" ]]; then
            source "$_root/.venv/bin/activate"
          fi

          unset _root
        '';
      };

      # Shell settings for CI.
      devShells.ci = pkgs.mkShellNoCC {
        packages = with pkgs; [
          bazel
          rust-toolchain # can be removed once all crates/ are built by Bazel
          prek
        ];

        shellHook = config.pre-commit.shellHook;
      };
    };
}
