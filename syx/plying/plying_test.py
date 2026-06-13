from syx import plying


def test_sum() -> None:
    assert plying.sum([1, 2, 3, 4]) == 10


def test_sum_empty() -> None:
    assert plying.sum([]) == 0
