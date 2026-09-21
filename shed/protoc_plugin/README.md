# protoc_plugin

## Examples

Run protoc-gen-message-descriptor-dump:

```bash
protoc \
--plugin=./examples/protoc-gen-message-descriptor-dump \
--message-descriptor-dump_out=. \
./testdata/example.proto
```
