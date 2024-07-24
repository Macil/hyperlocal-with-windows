use hyper::{
    body::{Body, Incoming},
    service::service_fn,
    Request, Response,
};
use hyper_util::rt::TokioIo;
use std::{future::Future, io, path::Path};
use tokio::net::UnixListener;

/// A cross-platform wrapper around a [`tokio::net::UnixListener`] or a Windows
/// equivalent. Using this type allows code using Unix sockets to be written
/// once and run on both Unix and Windows.
///
/// [`tokio::net::UnixListener`]:
///     https://docs.rs/tokio/1.39.1/tokio/net/struct.UnixListener.html
#[derive(Debug)]
pub struct CommonUnixListener(UnixListener);

impl CommonUnixListener {
    /// Open a Unix socket.
    ///
    /// # Errors
    ///
    /// This function will return any errors that occur while trying to open the
    /// provided path.
    pub fn bind(path: impl AsRef<Path>) -> io::Result<Self> {
        UnixListener::bind(path).map(Self)
    }
}

/// Extension trait for provisioning a hyper HTTP server over a Unix domain
/// socket.
///
/// # Example
///
/// ```rust
/// use hyper::Response;
/// use hyperlocal_with_windows::{
///     remove_unix_socket_if_present, CommonUnixListener, UnixListenerExt,
/// };
///
/// let future = async move {
///     let path = std::env::temp_dir().join("hyperlocal.sock");
///     remove_unix_socket_if_present(&path)
///         .await
///         .expect("removed any existing unix socket");
///     let listener = CommonUnixListener::bind(path).expect("parsed unix path");
///
///     listener
///         .serve(|| {
///             |_request| async {
///                 Ok::<_, hyper::Error>(Response::new("Hello, world.".to_string()))
///             }
///         })
///         .await
///         .expect("failed to serve a connection")
/// };
/// ```
pub trait UnixListenerExt {
    /// Indefinitely accept and respond to connections.
    ///
    /// Pass a function which will generate the function which responds to
    /// all requests for an individual connection.
    fn serve<MakeResponseFn, ResponseFn, ResponseFuture, B, E>(
        self,
        f: MakeResponseFn,
    ) -> impl Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
    where
        MakeResponseFn: Fn() -> ResponseFn,
        ResponseFn: Fn(Request<Incoming>) -> ResponseFuture,
        ResponseFuture: Future<Output = Result<Response<B>, E>>,
        B: Body + 'static,
        <B as Body>::Error: std::error::Error + Send + Sync,
        E: std::error::Error + Send + Sync + 'static;
}

impl UnixListenerExt for UnixListener {
    fn serve<MakeServiceFn, ResponseFn, ResponseFuture, B, E>(
        self,
        f: MakeServiceFn,
    ) -> impl Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
    where
        MakeServiceFn: Fn() -> ResponseFn,
        ResponseFn: Fn(Request<Incoming>) -> ResponseFuture,
        ResponseFuture: Future<Output = Result<Response<B>, E>>,
        B: Body + 'static,
        <B as Body>::Error: std::error::Error + Send + Sync,
        E: std::error::Error + Send + Sync + 'static,
    {
        async move {
            loop {
                let (stream, _) = self.accept().await?;
                let io = TokioIo::new(stream);

                let svc_fn = service_fn(f());

                hyper::server::conn::http1::Builder::new()
                    // On OSX, disabling keep alive prevents serve_connection from
                    // blocking and later returning an Err derived from E_NOTCONN.
                    .keep_alive(false)
                    .serve_connection(io, svc_fn)
                    .await?;
            }
        }
    }
}

impl UnixListenerExt for CommonUnixListener {
    fn serve<MakeServiceFn, ResponseFn, ResponseFuture, B, E>(
        self,
        f: MakeServiceFn,
    ) -> impl Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
    where
        MakeServiceFn: Fn() -> ResponseFn,
        ResponseFn: Fn(Request<Incoming>) -> ResponseFuture,
        ResponseFuture: Future<Output = Result<Response<B>, E>>,
        B: Body + 'static,
        <B as Body>::Error: std::error::Error + Send + Sync,
        E: std::error::Error + Send + Sync + 'static,
    {
        self.0.serve(f)
    }
}
