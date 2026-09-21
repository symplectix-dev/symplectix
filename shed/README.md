# shed

Somewhere to put experimental code.

Everything is `testonly`: depending on something for real
means moving it out.

Every BUILD file under `shed/` must include:

```python
package(
    default_testonly = True,
    default_visibility = ["//:__subpackages__"],
)
```

CI fails when `bazel query 'attr(testonly, 0, //shed/...)'` finds
a target that is not `testonly`.
