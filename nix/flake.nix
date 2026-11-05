{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/13e53762c5afaf384d4614e94527a9ccfafe41d0"; # 2026-11-04

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    git-hooks = {
      url = "github:cachix/git-hooks.nix";
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
        { pkgs, ... }:
        {
          formatter = pkgs.nixfmt-tree;
        };
    };
}
