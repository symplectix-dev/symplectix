"""Helper to run the executable built for fuzzing."""

import argparse
import os
import tempfile
from pathlib import Path
from typing import Final

_FUZZ_TARGET_SUFFIX: Final[str] = "_fuzz_target"


def _tmpdir() -> str:
    if tmpdir := os.getenv("TEST_TMPDIR"):
        return tmpdir
    return tempfile.TemporaryDirectory(prefix="fuzztest-", delete=False).name


def run(output_root: Path | None, fuzz_target: Path, fuzz_args: list[str]) -> None:
    """Execute fuzz_target from BUILD_WORKING_DIRECTORY."""
    # need to resolve before chdir because fuzz_target is passed as:
    # "$(rootpath :{}_fuzz_target)".format(name)
    fuzz_target = fuzz_target.resolve()
    fuzz_name = fuzz_target.name.removesuffix("_fuzz_target")

    # BUILD_WORKING_DIRECTORY is available only when `bazel run`.
    # In `bazel test`, chdir does not happen.
    if bwd := os.getenv("BUILD_WORKING_DIRECTORY"):
        os.chdir(bwd)

    if output_root is None:
        output_root = Path(_tmpdir())
    # artifact_prefix should be handled after chdir,
    # because output_root could be a relative path from BUILD_WORKING_DIRECTORY.
    artifact_prefix = output_root / fuzz_name / "artifact"
    artifact_prefix.mkdir(parents=True, exist_ok=True)

    os.execv(  # noqa: S606
        fuzz_target,
        [
            fuzz_name,
            f"-artifact_prefix={artifact_prefix}/",
            *fuzz_args,
        ],
    )


if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("--output_root", type=Path)
    p.add_argument("fuzz_target", type=Path)
    p.add_argument("fuzz_args", nargs="*")
    args = p.parse_args()
    run(**vars(args))
