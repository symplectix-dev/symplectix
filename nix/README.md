# nix/

Nix flake for the development environment.
Uses [flake-parts] to split the flake into
focused modules under `parts/`.

## Usage

```bash
# Enter dev shell
nix develop ./nix

# Format Nix files (run from nix/)
nix fmt

# Validate the flake. This does not run hooks; CI runs them with prek.
nix flake check ./nix
```

## Updating nixpkgs

```bash
gh api repos/nixos/nixpkgs/commits/nixpkgs-unstable \
  --jq '[.sha, .commit.committer.date[:10]] | join(" # ")'
```

Update the `nixpkgs.url` in `flake.nix`,
then run `nix flake update --flake ./nix`.

[flake-parts]: https://flake.parts
