from syx import plying


def test_sum() -> None:
    assert plying.sum([1, 2, 3, 4]) == 10


def test_sum_empty() -> None:
    assert plying.sum([]) == 0


def test_prod() -> None:
    assert plying.prod([2, 3, 4]) == 24


def test_prod_empty() -> None:
    assert plying.prod([]) == 1
