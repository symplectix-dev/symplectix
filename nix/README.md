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

# Run all checks (pre-commit hooks with --all-files + flake validation)
nix flake check ./nix
```

## Updating nixpkgs

```bash
gh api repos/nixos/nixpkgs/commits/nixpkgs-unstable \
  --jq '[.sha, .commit.committer.date[:10]] | join(" # ")'
```

Update the `nixpkgs.url` in `flake.nix`,
then run `nix flake update ./nix`.

[flake-parts]: https://flake.parts
