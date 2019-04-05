```
cargo +nightly build --release --target x86_64-unknown-linux-musl --out-dir ./target -Z unstable-options
```
- builds a release binary and places it in the target dir, which is where the Dockerfile will pick it up from