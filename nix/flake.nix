{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/767b0d3ec98a143ad9ed7dfc0d5553510ac27133"; # 2026-07-09

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
              })
            ];
          };
        };
    };
}
