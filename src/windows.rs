use std::{
    net::TcpStream,
    os::windows::prelude::{FromRawSocket, IntoRawSocket},
};
use uds_windows::UnixStream;

/// Tokio doesn't support `tokio::net::UnixStream` on Windows, so we convert a
/// `UnixStream` to a `std::net::TcpStream` which can be converted into a
/// `tokio::net::TcpStream`.
pub(crate) fn convert_unix_stream_to_nb_tcp_stream(stream: UnixStream) -> TcpStream {
    // We need to do this sometime before `tokio::net::TcpStream::from_std()` is
    // called.
    stream.set_nonblocking(true).unwrap();
    // Create a std::net::TcpStream from the raw Unix socket. Windows APIs that
    // accept sockets have defined behavior for Unix sockets (either they
    // successfully handle it or return an error), so this should be safe.
    unsafe { std::net::TcpStream::from_raw_socket(stream.into_raw_socket()) }
}
