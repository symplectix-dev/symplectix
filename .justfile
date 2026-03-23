default:
    @just --list --unsorted

# commit skipping pre-commit hooks
commit m:
    git commit --no-verify -m "{{m}}"

# amend skipping pre-commit hooks
amend:
    git commit --amend --no-verify
