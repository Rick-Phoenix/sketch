json-schema:
    cargo run --bin json-schema development

[working-directory('sketch')]
test:
    cargo test --all-features -- -q --nocapture

[working-directory('sketch')]
release-test version="patch":
    cargo release {{ version }}

[confirm]
[working-directory('sketch')]
release-exec version="patch":
    EXEC_RELEASE=true cargo release {{ version }} --execute

[working-directory('docs')]
docs:
    mdbook serve --open

install:
    cargo install --path ./sketch

install-dev:
    cargo install --path ./sketch --profile dev
