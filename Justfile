sync:
    cargo run -- --sync

web:
    cargo run -- --web

build-deb:
    cargo build --target=x86_64-unknown-linux-musl --release
    strip target/x86_64-unknown-linux-musl/release/onemorebeer-ui
    cargo deb --deb-revision="$(date +%s)" --target=x86_64-unknown-linux-musl
