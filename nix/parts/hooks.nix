{ ... }:
{
  perSystem = { pkgs, ... }: {
    # Disable the auto-generated checks.pre-commit output so that
    # `nix flake check` does not run prek. Hooks are run in CI via
    # `prek run --no-group no-ci` instead.
    pre-commit.check.enable = false;

    # Each hook declares its own package. They are intentionally not shared
    # with devShells.default so the two can evolve independently.
    pre-commit.settings = {
      src = ../../.;
      package = pkgs.prek;
      excludes = [
        "\\.pyi$"
        "/testdata/"
      ];
      # Run heavy checks at pre-push, not pre-commit. Since PRs are squash-merged,
      # intermediate commits do not need to pass all checks individually.
      default_stages = [ "pre-push" ];
      hooks = {
        no-commit-to-branch = {
          enable = true;
          stages = [ "pre-commit" ];
          raw.groups = [ "no-ci" ];
        };
        check-added-large-files = {
          enable = true;
          stages = [ "pre-commit" ];
        };
        check-case-conflicts = {
          enable = true;
          stages = [ "pre-commit" ];
        };
        check-merge-conflicts = {
          enable = true;
          stages = [ "pre-commit" ];
        };
        end-of-file-fixer = {
          enable = true;
          stages = [
            "pre-commit"
            "pre-push"
          ];
        };
        trim-trailing-whitespace = {
          enable = true;
          stages = [
            "pre-commit"
            "pre-push"
          ];
          args = [ "--markdown-linebreak-ext=md" ];
        };
        ruff = {
          enable = true;
          package = pkgs.ruff;
        };
        ruff-format = {
          enable = true;
          package = pkgs.ruff;
        };
        pyright = {
          enable = true;
          package = pkgs.basedpyright;
          entry = "${pkgs.basedpyright}/bin/basedpyright";
        };
        rustfmt = {
          enable = true;
          entry = "${pkgs.rust-toolchain}/bin/rustfmt";
          pass_filenames = true;
        };
        shellcheck = {
          enable = true;
          args = [ "--severity=warning" ];
          excludes = [ "\\.envrc$" ];
        };
        shfmt = {
          enable = true;
          settings.indent = 2;
          settings.language-dialect = "bash";
        };
        clang-format = {
          enable = true;
          types_or = [ "proto" ];
        };
        buildifier = {
          enable = true;
          name = "buildifier";
          entry = "${pkgs.bazelisk}/bin/bazelisk run //bazel:buildifier";
          types = [ "bazel" ];
          language = "system";
          pass_filenames = false;
        };
        typos = {
          enable = true;
          stages = [ "manual" ];
          raw.groups = [ "no-ci" ];
        };
        zizmor = {
          enable = true;
          package = pkgs.zizmor;
          args = [ "--persona" "pedantic" ];
          stages = [ "manual" ];
          raw.groups = [ "no-ci" ];
        };
      };
    };
  };
}
