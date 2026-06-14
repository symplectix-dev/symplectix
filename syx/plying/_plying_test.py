from syx.plying import _plying  # pyright: ignore[reportPrivateUsage]


def test_sum() -> None:
    assert _plying.sum([1, 2, 3, 4]) == 10


def test_sum_empty() -> None:
    assert _plying.sum([]) == 0


def test_prod() -> None:
    assert _plying.prod([2, 3, 4]) == 24


def test_prod_empty() -> None:
    assert _plying.prod([]) == 1
