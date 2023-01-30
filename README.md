# inotify-guest

build this binary

```bash
cargo build --bin main --features="tokio/rt-multi-thread main tokio/macros" --release
```

And you can find in `target/release/main`