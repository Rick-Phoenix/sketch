json-schema:
    cargo run --bin json-schema unstable

test:
    cargo test --all-features -- -q --nocapture

release-test version="patch":
    cargo release {{ version }}

[confirm]
release-exec $EXEC_RELEASE="true" version="patch":
    cargo release {{ version }} --execute
