{ ... }:
{
  perSystem = { pkgs, ... }: {
    # Disable the auto-generated checks.pre-commit output so that
    # `nix flake check` does not run prek. Hooks are run in CI via
    # `prek run --no-group no-ci` instead.
    pre-commit.check.enable = false;

    # Each hook declares its own package. They are intentionally not shared
    # with devShells.default so the two can evolve independently.
    pre-commit.settings =
      let
        withDefaults = defs: builtins.mapAttrs (_: v: v // defs);
        alwaysEnabled = {
          enable = true;
        };
        nixManaged = {
          language = "system";
        };

        mkBuiltinHooks = withDefaults alwaysEnabled;
        mkBuiltinHooksGroup = priority: withDefaults (alwaysEnabled // { raw.priority = priority; });

        mkCustomHooks = withDefaults (alwaysEnabled // nixManaged);

        builtinHooks =
          mkBuiltinHooksGroup 1 {
            no-commit-to-branch = {
              stages = [ "pre-commit" ];
              raw.groups = [ "no-ci" ];
            };
            check-added-large-files = {
              stages = [ "pre-commit" ];
            };
            check-case-conflicts = {
              stages = [ "pre-commit" ];
            };
            check-merge-conflicts = {
              stages = [ "pre-commit" ];
            };
          }
          // mkBuiltinHooks {
            end-of-file-fixer = {
              stages = [
                "pre-commit"
                "pre-push"
              ];
            };
            trim-trailing-whitespace = {
              stages = [
                "pre-commit"
                "pre-push"
              ];
              args = [ "--markdown-linebreak-ext=md" ];
            };
            ruff = {
              package = pkgs.ruff;
            };
            ruff-format = {
              package = pkgs.ruff;
            };
            pyright = {
              package = pkgs.basedpyright;
              entry = "${pkgs.basedpyright}/bin/basedpyright";
            };
            rustfmt = {
              entry = "${pkgs.rust-toolchain}/bin/rustfmt";
              pass_filenames = true;
            };
            shellcheck = {
              args = [ "--severity=warning" ];
              excludes = [ "\\.envrc$" ];
            };
            shfmt = {
              settings.indent = 2;
              settings.language-dialect = "bash";
            };
            clang-format = {
              types_or = [ "proto" ];
            };
            typos = {
              stages = [ "manual" ];
              raw.groups = [ "no-ci" ];
            };
            zizmor = {
              args = [
                "--persona"
                "pedantic"
              ];
              stages = [ "manual" ];
              raw.groups = [ "no-ci" ];
            };
          };

        customHooks = mkCustomHooks {
          buildifier = {
            name = "buildifier";
            entry = "${pkgs.bazelisk}/bin/bazelisk run //bazel:buildifier";
            types = [ "bazel" ];
            pass_filenames = false;
          };
        };
      in
      {
        src = ../../.;
        package = pkgs.prek;
        excludes = [
          "\\.pyi$"
          "/testdata/"
        ];
        # Run heavy checks at pre-push, not pre-commit. Since PRs are squash-merged,
        # intermediate commits do not need to pass all checks individually.
        default_stages = [ "pre-push" ];
        hooks = builtinHooks // customHooks;
      };
  };
}
