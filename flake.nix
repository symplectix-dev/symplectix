{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/4724d5647207377bede08da3212f809cbd94a648"; # 2026-03-23
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self, ... }@inputs:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forEachSupportedSystem =
        f:
        inputs.nixpkgs.lib.genAttrs supportedSystems (
          system:
          f {
            pkgs = import inputs.nixpkgs {
              inherit system;
              # if you need unfree packages:
              # config.allowUnfree = true;
              overlays = [
                inputs.self.overlays.default
              ];
            };
          }
        );
    in
    {
      overlays.default = final: prev: {
        rust-toolchain =
          with inputs.fenix.packages.${prev.stdenv.hostPlatform.system};
          combine ([
            stable.clippy
            stable.rustc
            stable.rust-src
            stable.cargo
            complete.rustfmt
          ]);
      };

      devShells = forEachSupportedSystem (
        { pkgs }:
        let
          python = pkgs.python314;
          bazelisk-as-bazel = pkgs.writeShellScriptBin "bazel" ''
            exec ${pkgs.bazelisk}/bin/bazelisk $@
          '';
        in
        {
          default = pkgs.mkShellNoCC {
            packages = with pkgs; [
              # Bazel
              bazelisk
              bazelisk-as-bazel
              bazel-buildtools
              # GitHub client
              github-cli
              # rust
              rust-toolchain
              rust-analyzer
              # python
              basedpyright
              python.pkgs.ruff
              # protobuf
              protobuf
              # nix
              nixfmt-tree
              # lint for .github/workflows
              zizmor
              # pre commit
              prek
            ];

            env = {
              RUST_SRC_PATH = "${pkgs.rust-toolchain}/lib/rustlib/src/rust/library";
            };

            shellHook = ''
              # Anchor to the repo root so paths are correct even when entering the shell
              # from a different working directory (e.g., `nix develop path/to/symplectix`).
              _root=$(git rev-parse --show-toplevel)

              # Create venv on first entry.
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
        }
      );
    };
}
