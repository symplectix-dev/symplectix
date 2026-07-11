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

        withDefaults = defs: builtins.mapAttrs (_: v: defs // v);
        mkHooks = groups: withDefaults (alwaysEnabled // { raw.groups = groups; });

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

        # Prefer `bazel test` over hooks. Bazel provides consistent coverage
        # across the dependency graph, and its caching makes repeated runs fast.
        # Hooks are for lightweight checks or tools not (yet) integrated into
        # Bazel. But of course a hook is better than no check at all: if Bazel
        # integration is too costly, adding a hook is the right pragmatic choice.
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
          // mkHooksWithPriority 10 [ "ci" "format" ] {
            shfmt = {
              settings.indent = 2;
              settings.language-dialect = "bash";
            };
            ruff-format = noSettings;
            buf-format = {
              name = "buf-format";
              entry = "${pkgs.buf}/bin/buf format -w";
              types = [ "proto" ];
            };
            rustfmt = {
              entry = "${pkgs.rust-toolchain}/bin/rustfmt";
              pass_filenames = true;
            };
          }
          // mkHooksWithPriority 20 [ "ci" "lint" ] {
            shellcheck = {
              args = [ "--severity=warning" ];
              excludes = [ "\\.envrc$" ];
            };
            ruff = {
              name = "ruff-check";
              entry = "${pkgs.ruff}/bin/ruff check --diff";
            };
            pyright-all = {
              name = "pyright-all";
              package = pkgs.basedpyright;
              entry = "${pkgs.basedpyright}/bin/basedpyright";
              pass_filenames = false;
              files = "\\.py$";
              types = [ "python" ];
              stages = [ "manual" ];
            };
          }
          // mkHooksWithPriority 20 [ "no-ci" "lint" ] {
            # TODO: Only changed files are checked. If a type change in one
            # module breaks a dependent module, that breakage goes undetected.
            # Use pyright-all (pass_filenames = false) or move to Bazel.
            pyright = {
              package = pkgs.basedpyright;
              entry = "${pkgs.basedpyright}/bin/basedpyright";
            };
          };
      };
  };
}
