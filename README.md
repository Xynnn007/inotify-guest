# QGS-Multiplexer

This tool is used to multiplex QGS to serve many TD-Guests created by dragonball.
The tool will watch `/var/lib/vc/dragonball/<guest-id>/root/kata.hvsock_`.
If a new one is created, there will be a new connection between the socket file and the QGS' socketfile `/var/run/tdx-qgs/qgs.socket` by default.

build this binary

```bash
cargo build --bin qgs-multiplexer --features="tokio/rt-multi-thread main tokio/macros" --release
```

And you can find in `target/release/qgs-multiplexer`