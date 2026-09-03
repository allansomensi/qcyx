set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

[group('test')]
test:
    @cargo test

[group('test')]
test-watch:
    @cargo watch -q -c -x test

[group('check')]
lint:
    @cargo fmt --all -- --check
    @cargo clippy --all --all-targets --all-features -- --deny warnings

[group('check')]
lint-fix:
    @cargo fmt --all
    @cargo clippy --all --all-targets --all-features -- --deny warnings

[group('dev')]
gui:
    @cargo run --bin qcyx

[group('dev')]
cli *ARGS:
    @cargo run --bin qcyx-cli -- {{ ARGS }}

[group('docs')]
docs CRATE:
    @open "https://docs.rs/{{ CRATE }}"

[group('misc')]
clean:
    @cargo clean
