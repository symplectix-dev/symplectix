# protoc_plugin

One plugin to a package, alongside the `proto.plugin` target naming it.
`testdata/` holds the proto and runs every plugin over it.

## Example

Dump the message descriptors of `testdata/customer.proto`:

```bash
bazel build //shed/protoc_plugin/testdata:customer_message_descriptor_dump
```

Generate Arrow schemas for the same messages:

```bash
bazel build //shed/protoc_plugin/testdata:customer_arrow_schema
```
