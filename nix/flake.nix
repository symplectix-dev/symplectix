{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/9eac87a12312b8f60dd52e1c6e1a265f6fc7f5fc"; # 2026-06-14

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      imports = [
        inputs.git-hooks.flakeModule
        ./parts/hooks.nix
        ./parts/shell.nix
      ];
      perSystem =
        { system, pkgs, ... }:
        {
          formatter = pkgs.nixfmt-tree;
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            # if you need unfree packages:
            # config.allowUnfree = true;
            overlays = [
              (final: prev: {
                rust-toolchain =
                  with inputs.fenix.packages.${system};
                  combine [
                    stable.clippy
                    stable.rustc
                    stable.rust-src
                    stable.cargo
                    complete.rustfmt
                  ];
                # Overlay prek to 0.4.5 for the groups feature (nixpkgs has 0.3.11).
                # Remove this overlay once nixpkgs ships prek >= 0.4.5.
                #   nix eval "<nixpkgs.url>#prek.version"
                prek =
                  let
                    version = "0.4.5";
                    # hash: SRI form of the per-archive checksum on the release page, e.g.:
                    # nix hash convert sha256:$(curl -Ls https://github.com/j178/prek/releases/download/v0.4.5/prek-x86_64-unknown-linux-musl.tar.gz.sha256 | awk '{print $1}')
                    binaries = {
                      x86_64-linux = {
                        archive = "prek-x86_64-unknown-linux-musl.tar.gz";
                        hash = "sha256-lGmNHNdOe0YqAXJX7+q29GFC9Ps9qaExTwbB1Vwm1hQ=";
                      };
                      aarch64-linux = {
                        archive = "prek-aarch64-unknown-linux-musl.tar.gz";
                        hash = "sha256-nD8GhrbxKZWy+cvClTpFeYdiKim7zeqmT2JSevhR/B4=";
                      };
                      aarch64-darwin = {
                        archive = "prek-aarch64-apple-darwin.tar.gz";
                        hash = "sha256-7Ukgdi8OPbBxY8Q3r2IqHUkF4a1wU2kqSVVRuc6SVJ8=";
                      };
                    };
                  in
                    prev.stdenv.mkDerivation {
                      pname = "prek";
                      inherit version;
                      src = prev.fetchurl {
                        url = "https://github.com/j178/prek/releases/download/v${version}/${binaries.${system}.archive}";
                        hash = binaries.${system}.hash;
                      };
                      installPhase = ''
                        install -Dm755 prek $out/bin/prek
                      '';
                      meta.mainProgram = "prek";
                    };
              })
            ];
          };
        };
    };
}
