"""Constructs a math.comb table."""

from math import comb

from pydantic import BaseModel


class _Comb(BaseModel):
    table: list[list[int]]


def _table(size: int) -> _Comb:
    table = [[0 for _ in range(size)] for _ in range(size)]
    for n in range(size):
        for k in range(size):
            table[n][k] = comb(n, k)
    return _Comb(table=table)


if __name__ == "__main__":
    import argparse
    from pathlib import Path

    p = argparse.ArgumentParser()
    p.add_argument("--size", type=int, default=32)
    p.add_argument("--path", type=Path)
    args = p.parse_args()

    with args.path.open(mode="w") as f:
        f.write(_table(args.size).model_dump_json())
