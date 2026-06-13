import syx.plying


def test_sum() -> None:
    assert syx.plying.sum([1, 2, 3, 4]) == 10


def test_sum_empty() -> None:
    assert syx.plying.sum([]) == 0
