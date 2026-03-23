{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    fenix.url = "github:nix-community/fenix/monthly";
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
        in
        {
          default = pkgs.mkShellNoCC {
            packages = with pkgs; [
              # rust
              rust-toolchain
              rust-analyzer
              # python
              python
              basedpyright
              python.pkgs.uv
              python.pkgs.ruff
              # protobuf
              protobuf
              # nix
              nixfmt-tree
              # lint for .github/workflows
              zizmor
              # command runner
              just
              # pre commit
              prek
            ];

            env = {
              RUST_SRC_PATH = "${pkgs.rust-toolchain}/lib/rustlib/src/rust/library";
            };
          };
        }
      );
    };
}
