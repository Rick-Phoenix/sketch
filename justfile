json-schema:
    cargo run --bin json-schema development

test:
    cargo test --all-features -p sketch -- -q --nocapture

[working-directory('sketch')]
release-test version="patch":
    cargo release {{ version }}

[confirm]
[working-directory('sketch')]
release-exec version="patch":
    EXEC_RELEASE=true cargo release {{ version }} --execute

[working-directory('docs')]
open-docs:
    mdbook serve --open
