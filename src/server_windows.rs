use std::{io, path::Path};

use hyper::server::{Builder, Server};

use conn::SocketIncoming;

pub(crate) mod conn {
    use hyper::server::accept::Accept;
    use pin_project_lite::pin_project;
    use std::{
        io,
        path::Path,
        pin::Pin,
        task::{Context, Poll},
        thread,
    };
    use tokio::sync::mpsc;
    use uds_windows::UnixListener;

    use crate::windows::convert_unix_stream_to_nb_tcp_stream;

    pin_project! {
        /// A stream of connections from binding to a socket.
        #[derive(Debug)]
        pub struct SocketIncoming {
            rx: mpsc::Receiver<std::net::TcpStream>,
        }
    }

    impl SocketIncoming {
        /// Creates a new `SocketIncoming` binding to provided socket path.
        ///
        /// # Errors
        /// Refer to [`tokio::net::Listener::bind`](https://docs.rs/tokio/1.15.0/tokio/net/struct.UnixListener.html#method.bind).
        pub fn bind(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
            let listener = UnixListener::bind(path)?;

            Ok(Self::from_listener(listener))
        }

        /// Creates a new `SocketIncoming` from Tokio's `UnixListener`
        ///
        /// ```rust,ignore
        /// let socket = SocketIncoming::from_listener(unix_listener);
        /// let server = Server::builder(socket).serve(service);
        /// ```
        #[allow(clippy::must_use_candidate)]
        pub fn from_listener(listener: UnixListener) -> Self {
            let (tx, rx) = mpsc::channel(32);

            // TODO We aren't fully handling closing the socket. Ideally when the
            // SocketIncoming is dropped, we would abort the current accept()
            // call and then close the socket. Currently we only close
            // the socket once we receive a connection after the SocketIncoming was dropped.
            thread::spawn(move || {
                while let Ok((socket, _addr)) = listener.accept() {
                    let tcp_stream = convert_unix_stream_to_nb_tcp_stream(socket);
                    // Give this TcpStream to Hyper.
                    if tx.blocking_send(tcp_stream).is_err() {
                        // End if the receiver closed.
                        break;
                    }
                }
            });

            Self { rx }
        }
    }

    impl Accept for SocketIncoming {
        type Conn = tokio::net::TcpStream;
        type Error = io::Error;

        fn poll_accept(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
            self.project()
                .rx
                .poll_recv(cx)
                .map(|t| t.map(tokio::net::TcpStream::from_std))
        }
    }

    impl From<UnixListener> for SocketIncoming {
        fn from(listener: UnixListener) -> Self {
            Self::from_listener(listener)
        }
    }
}

/// Extension trait for provisioning a hyper HTTP server over a Unix domain
/// socket.
///
/// # Example
///
/// ```rust
/// use hyper::{Server, Body, Response, service::{make_service_fn, service_fn}};
/// use hyperlocal_with_windows::UnixServerExt;
///
/// # async {
/// let make_service = make_service_fn(|_| async {
///     Ok::<_, hyper::Error>(service_fn(|_req| async {
///         Ok::<_, hyper::Error>(Response::new(Body::from("It works!")))
///     }))
/// });
///
/// Server::bind_unix("/tmp/hyperlocal.sock")?.serve(make_service).await?;
/// # Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
/// # };
/// ```
pub trait UnixServerExt {
    /// Convenience method for constructing a Server listening on a Unix socket.
    #[allow(clippy::missing_errors_doc)]
    fn bind_unix(path: impl AsRef<Path>) -> Result<Builder<SocketIncoming>, io::Error>;
}

impl UnixServerExt for Server<SocketIncoming, ()> {
    fn bind_unix(path: impl AsRef<Path>) -> Result<Builder<SocketIncoming>, io::Error> {
        let incoming = SocketIncoming::bind(path)?;
        Ok(Server::builder(incoming))
    }
}
