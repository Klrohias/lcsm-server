use anyhow::Result;
use bytes::BytesMut;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    sync::{broadcast, mpsc},
};

pub type BinarySequence = Vec<u8>;

pub async fn redirect_output(
    mut output: impl AsyncRead + Unpin,
    sender: broadcast::Sender<BinarySequence>,
) -> Result<()> {
    let mut buf = BytesMut::with_capacity(1024);

    loop {
        match output.read(&mut buf).await? {
            0 => break,
            n => {
                sender.send(buf[..n].to_vec())?;
            }
        }
    }

    Ok(())
}

pub async fn redirect_input(
    mut input: impl AsyncWrite + Unpin,
    mut receiver: mpsc::Receiver<BinarySequence>,
) -> Result<()> {
    loop {
        match receiver.recv().await {
            Some(it) => {
                input.write(&it).await?;
            }
            None => break,
        }
    }

    Ok(())
}
