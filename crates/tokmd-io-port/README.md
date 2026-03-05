# tokmd-io-port

I/O port traits for host-abstracted file access.

Provides `ReadFs` – a trait that abstracts read-only filesystem operations so
that `tokmd-scan` (and other crates) can work against in-memory data when
compiled for WASM targets.

## Implementations

| Struct   | Purpose                            |
|----------|------------------------------------|
| `HostFs` | Delegates to `std::fs` (default)   |
| `MemFs`  | In-memory store for tests and WASM |

## License

MIT OR Apache-2.0
