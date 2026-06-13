# ruff: noqa
# This file is auto-generated. DO NOT EDIT MANUALLY.
"""
Python bindings for the plying crate.

Exposes composable left fold operations over sequences of integers.
"""

from collections.abc import Sequence

def prod(items: Sequence[int]) -> int:
    """
    Return the product of all items.
    
    # Arguments
    
    * `items` - A sequence of integers.
    
    # Examples
    
    ```python
    assert prod([2, 3, 4]) == 24
    assert prod([]) == 1
    ```
    """

def sum(items: Sequence[int]) -> int:
    """
    Return the sum of all items.
    
    # Arguments
    
    * `items` - A sequence of integers.
    
    # Examples
    
    ```python
    assert sum([1, 2, 3]) == 6
    assert sum([]) == 0
    ```
    """
