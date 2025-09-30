json-schema:
    cargo run --bin json-schema development

test: json-schema
    cargo test --all-features -- -q --nocapture

[working-directory('sketch')]
release-test version="patch":
    cargo release {{ version }}

[confirm]
[working-directory('sketch')]
release-exec $EXEC_RELEASE="true" version="patch":
    cargo release {{ version }} --execute
