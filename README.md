# http-load-tester

A small command line tool that sends HTTP requests to a URL with configurable method, request count, concurrency, headers, and body.

## Install Rust and Cargo

Install Rust with [rustup](https://rustup.rs/). Cargo, Rust's package manager and build tool, is installed along with it. Follow the installer instructions, then open a new terminal and check the installation:

```sh
rustc --version
cargo --version
```

## Install for use from anywhere

From the project directory, install the executable into Cargo's binary directory:

```sh
cargo install --path .
```

Cargo's binary directory (usually `~/.cargo/bin`) must be on your `PATH`. After installation, run it from any directory:

```sh
http-load-tester --help
```

## Run

You can run it through Cargo:

```sh
cargo run --release -- \
  --url https://example.com \
  --method get \
  --request-count 100 \
  --concurrency 10
```

Or invoke the built executable directly:

```sh
./target/release/http-load-tester \
  --url https://example.com \
  --method get \
  --request-count 100 \
  --concurrency 10
```

Or use the global installation mentioned previously:

```sh
http-load-tester \
  --url https://example.com \
  --method get \
  --request-count 100 \
  --concurrency 10
```

Use `--help` to see all available options. For example, add request headers by repeating `--headers NAME:VALUE`, and provide a request body with `--body`:

```sh
cargo run --release -- \
  --url https://example.com/api \
  --method post \
  --request-count 20 \
  --concurrency 5 \
  --headers 'Content-Type: application/json' \
  --body '{"message":"hello"}'
```

Supported methods are `get`, `post`, `put`, `patch`, and `delete`. Request count and concurrency must both be positive integers. Add `--debug` to print each request's result and response body.

## Build

From the project directory, build the executable:

```sh
cargo build --release
```

The executable will be at `target/release/http-load-tester` (on Windows, `target\release\http-load-tester.exe`).
