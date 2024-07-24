<div align="center">
🔌 ✨

</div>

<h1 align="center">
  hyperlocal-with-windows
</h1>

<p align="center">
   <a href="https://github.com/hyperium/hyper">Hyper</a> client and server bindings for <a href="https://github.com/tokio-rs/tokio/tree/master/tokio-net/src/uds/">Unix domain sockets</a>, with Windows support
</p>

<div align="center">
  <a alt="GitHub Actions" href="https://github.com/Macil/hyperlocal-with-windows/actions">
    <img src="https://github.com/Macil/hyperlocal-with-windows/workflows/Main/badge.svg"/>
  </a>
  <a alt="crates.io" href="https://crates.io/crates/hyperlocal-with-windows">
    <img src="https://img.shields.io/crates/v/hyperlocal-with-windows.svg?logo=rust"/>
  </a>
  <a alt="docs.rs" href="http://docs.rs/hyperlocal-with-windows">
    <img src="https://docs.rs/hyperlocal-with-windows/badge.svg"/>
  </a>
  <a alt="latest docs" href="https://macil.github.io/hyperlocal-with-windows">
   <img src="https://img.shields.io/badge/docs-latest-green.svg"/>
  </a>
  <a alt="license" href="LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-brightgreen.svg"/>
  </a>
</div>

<br />

Hyper is a rock solid [Rust](https://www.rust-lang.org/) HTTP client and server toolkit.
[Unix domain sockets](https://en.wikipedia.org/wiki/Unix_domain_socket) provide a mechanism
for host-local interprocess communication. `hyperlocal-with-windows` builds on and complements Hyper's
interfaces for building Unix domain socket HTTP clients and servers.

This is useful for exposing simple HTTP interfaces for your Unix daemons in cases where you
want to limit access to the current host, in which case, opening and exposing tcp ports is
not needed. Examples of Unix daemons that provide this kind of host local interface include
[Docker](https://docs.docker.com/engine/misc/), a process container manager.

This library is a fork of [hyperlocal](https://github.com/softprops/hyperlocal) with Windows support added. This project is not Windows-specific; it is cross-platform. The Windows support has a limitation: when acting as a server and listening on a Unix socket, the underlying socket may remain open until the program exits even if the server is shut down. This is not expected to be a problem for usual programs that listen on a Unix socket until the program exits, but this may be a problem for other use cases. This library will be discontinued once Windows support [is added upstream into hyperlocal](https://github.com/softprops/hyperlocal/issues/21).

## Installation

Add the following to your `Cargo.toml` file

```toml
[dependencies]
hyperlocal-with-windows = "0.9"
```

## Usage

### Servers

A typical server can be built by creating a `tokio::net::UnixListener` and accepting connections in a loop using
`hyper::service::service_fn` to create a request/response processing function, and connecting the `UnixStream` to it
using `hyper::server::conn::http1::Builder::new().serve_connection()`.

`hyperlocal` provides an extension trait `UnixListenerExt` with an implementation of this.

An example is at [examples/server.rs](./examples/server.rs), runnable via `cargo run --example server`

To test that your server is working you can use an out-of-the-box tool like `curl`

```sh
$ curl --unix-socket /tmp/hyperlocal.sock localhost

It's a Unix system. I know this.
```

### Clients

`hyperlocal` also provides bindings for writing unix domain socket based HTTP clients the `Client` interface from the
`hyper-utils` crate.

An example is at [examples/client.rs](./examples/client.rs), runnable via `cargo run --example client`

Hyper's client interface makes it easy to send typical HTTP methods like `GET`, `POST`, `DELETE` with factory
methods, `get`, `post`, `delete`, etc. These require an argument that can be transformed into a `hyper::Uri`.

Since Unix domain sockets aren't represented with hostnames that resolve to ip addresses coupled with network ports,
your standard over the counter URL string won't do. Instead, use a `hyperlocal_with_windows::Uri`, which represents both file path to the domain
socket and the resource URI path and query string.

---

Doug Tangren (softprops) 2015-2024
