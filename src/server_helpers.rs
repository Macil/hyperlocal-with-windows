use std::{io, path::Path};

/// Helper function to delete a Unix socket if it exists.
/// This function correctly handles Unix sockets on Windows which need to be
/// handled specially.
///
/// # Errors
/// Refer to [`tokio::fs::remove_file`](https://docs.rs/tokio/latest/tokio/fs/fn.remove_file.html).
pub async fn remove_unix_socket_if_present(path: impl AsRef<Path>) -> io::Result<()> {
    // We need to delete the unix socket if it exists so we can listen on a
    // new one.
    // On Windows, metadata() returns an Uncategorized error if the file
    // exists as a unix socket.
    let must_delete = match tokio::fs::metadata(&path).await {
        Ok(_) => true,
        Err(err) => err.kind() != std::io::ErrorKind::NotFound,
    };
    if must_delete {
        tokio::fs::remove_file(&path).await?;
    }
    Ok(())
}
