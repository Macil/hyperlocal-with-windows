use hyper::Response;
use hyperlocal_with_windows::{remove_unix_socket_if_present, CommonUnixListener, UnixListenerExt};
use std::error::Error;

const PHRASE: &str = "It's a Unix system. I know this.\n";

// Adapted from https://hyper.rs/guides/1/server/hello-world/
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let path = std::env::temp_dir().join("hyperlocal.sock");

    remove_unix_socket_if_present(&path).await?;

    println!("Listening for connections at {}.", path.display());

    CommonUnixListener::bind(path)?
        .serve(|| {
            println!("Accepted connection.");

            |_request| async {
                let body = PHRASE.to_string();
                Ok::<_, hyper::Error>(Response::new(body))
            }
        })
        .await?;

    Ok(())
}
