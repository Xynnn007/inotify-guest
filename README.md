# QGS-Multiplexer

This tool is used to multiplex QGS to serve many TD-Guests created by dragonball.
The tool will watch `/var/lib/vc/dragonball`. Whenever a new guest is created, 
If a new one is created, there will be a new connection between the socket file and the QGS' socketfile `/var/run/tdx-qgs/qgs.socket` by default.

build this binary

```bash
cargo build --bin qgs-multiplexer --features="tokio/rt-multi-thread main tokio/macros" --release
```

And you can find in `target/release/qgs-multiplexer`.

By default, it will use `"/var/run/tdx-qgs/qgs.socket"` as the qgs socket.