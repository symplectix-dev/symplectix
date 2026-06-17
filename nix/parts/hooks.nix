{ ... }:
{
  perSystem = { pkgs, ... }: {
    # Disable the auto-generated checks.pre-commit output so that
    # `nix flake check` does not run prek. Hooks are run in CI via
    # `prek run --no-group no-ci` instead.
    pre-commit.check.enable = false;

    pre-commit.settings =
      let
        noSettings = { };
        alwaysEnabled = {
          enable = true;
        };
        nixManaged = alwaysEnabled // {
          language = "system";
        };

        withDefaults = defs: builtins.mapAttrs (_: v: defs // v);

        # Hooks that share the same priority value run concurrently,
        # subject to the global concurrency limit.
        #
        # If two hooks run in the same priority group and both mutate the same files
        # (or depend on shared state), results are undefined.
        # Use separate priorities to avoid overlap.
        #
        # require_serial = true limits a hook to one worker at a time, but other
        # hooks at the same priority still run alongside it. To isolate a hook
        # from all others, give it a unique priority.
        #
        # https://prek.j178.dev/reference/configuration/#priority
        mkHooksWithPriority =
          priority: groups:
          withDefaults (
            alwaysEnabled
            // {
              raw.priority = priority;
              raw.groups = groups;
            }
          );

        mkHooks = groups: withDefaults (alwaysEnabled // { raw.groups = groups; });
        mkCustomHooks = groups: withDefaults (nixManaged // { raw.groups = groups; });
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
        hooks =
          mkHooks [ "ci" ] {
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
            zizmor = {
              args = [
                "--persona"
                "pedantic"
              ];
              stages = [ "manual" ];
            };
          }
          // mkHooks [ "no-ci" ] {
            typos = {
              stages = [ "manual" ];
            };
          }
          // mkHooksWithPriority 10 [ "no-ci" ] {
            no-commit-to-branch = {
              stages = [ "pre-commit" ];
            };
          }
          // mkHooksWithPriority 10 [ "ci" ] {
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
          // mkHooksWithPriority 10 [ "ci" ] {
            shellcheck = {
              args = [ "--severity=warning" ];
              excludes = [ "\\.envrc$" ];
            };
            ruff = {
              entry = "${pkgs.ruff}/bin/ruff check --diff";
            };
            pyright = {
              package = pkgs.basedpyright;
              entry = "${pkgs.basedpyright}/bin/basedpyright";
              pass_filenames = false;
            };
          }
          // mkHooksWithPriority 20 [ "ci" ] {
            shfmt = {
              settings.indent = 2;
              settings.language-dialect = "bash";
            };
            ruff-format = noSettings;
            clang-format = {
              types_or = [ "proto" ];
            };
            rustfmt = {
              entry = "${pkgs.rust-toolchain}/bin/rustfmt";
              pass_filenames = true;
            };
          }
          // mkCustomHooks [ "ci" ] {
            buildifier = {
              name = "buildifier";
              entry = "${pkgs.bazelisk}/bin/bazelisk run //bazel:buildifier";
              types = [ "bazel" ];
              pass_filenames = false;
            };
          };
      };
  };
}
