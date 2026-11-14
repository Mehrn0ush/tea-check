# tea-check

CLI tool that fetches TEA product release JSON from one or more URLs and prints a structured report.

## Usage

```bash
cargo run -- https://www.vulnetix.com/tea/v1.0.0/productRelease/71286087-babf-5795-a6ce-4c350bc944b4  
cargo run -- url1 url2 url3
```

## Features

- Concurrent requests via `tokio` + `futures`
- Deserialization with `libtea-core`
- Reports identifiers, components, artifacts, formats, and checksums
- Graceful errors with `anyhow` (exit code 1 on failure)

## Dependencies

`tokio` · `reqwest` · `futures` · `libtea-core` · `anyhow`
