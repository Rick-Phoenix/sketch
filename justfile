json-schema:
    cargo run --bin json-schema development

[working-directory('sketch')]
test:
    cargo test -p sketch-it -- -q --nocapture

generate-cli-docs:
    cargo test -p sketch-it --features schemars generate_cli_docs

[working-directory('sketch')]
test-all:
    cargo test -p sketch-it --all-features -- -q --nocapture

[working-directory('docs')]
docs: generate-cli-docs
    mdbook serve --open

install:
    cargo install --path ./sketch

install-dev:
    cargo install --path ./sketch --profile dev

release version exec="": test-all
    ./pre_release.sh {{ version }} {{ exec }}
    cargo release {{ version }} -p sketch-it {{ exec }}
