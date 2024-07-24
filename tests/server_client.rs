use http_body_util::{BodyExt, Full};
use std::error::Error;

use hyper::{body::Bytes, Response};
use hyper_util::client::legacy::Client;
use hyperlocal_with_windows::{
    remove_unix_socket_if_present, CommonUnixListener, UnixClientExt, UnixConnector,
    UnixListenerExt, Uri,
};

const PHRASE: &str = "It works!";

#[tokio::test]
async fn test_server_client() -> Result<(), Box<dyn Error + Send + Sync>> {
    let path = std::env::temp_dir().join("hyperlocal.sock");

    remove_unix_socket_if_present(&path).await?;

    let listener = CommonUnixListener::bind(&path)?;

    let _server_task = tokio::spawn(async move {
        listener
            .serve(|| |_req| async { Ok::<_, hyper::Error>(Response::new(PHRASE.to_string())) })
            .await
    });

    let client: Client<UnixConnector, Full<Bytes>> = Client::unix();

    let url = Uri::new(&path, "/").into();

    let mut response = client.get(url).await?;
    let mut bytes = Vec::default();

    while let Some(frame_result) = response.frame().await {
        let frame = frame_result?;

        if let Some(segment) = frame.data_ref() {
            bytes.extend(segment.iter().as_slice());
        }
    }

    let string = String::from_utf8(bytes)?;

    assert_eq!(PHRASE, string);

    Ok(())
}
