use hyper::{
    service::{make_service_fn, service_fn},
    Body, Response, Server,
};
use hyperlocal_with_windows::{remove_unix_socket_if_present, UnixServerExt};
use std::{error::Error, fs, path::Path};

const PHRASE: &str = "It's a Unix system. I know this.";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let path = Path::new("/tmp/hyperlocal.sock");

    remove_unix_socket_if_present(&path).await?;

    let make_service = make_service_fn(|_| async {
        Ok::<_, hyper::Error>(service_fn(|_req| async {
            Ok::<_, hyper::Error>(Response::new(Body::from(PHRASE)))
        }))
    });

    Server::bind_unix(path)?.serve(make_service).await?;

    Ok(())
}
