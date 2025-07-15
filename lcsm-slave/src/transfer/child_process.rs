use std::sync::Arc;

use anyhow::Result;
use bytes::BytesMut;

use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    process::Child,
    sync::{RwLock, broadcast, mpsc},
};

use crate::services::ProcessState;

pub type BinarySequence = Vec<u8>;

pub async fn redirect_output(
    mut output: impl AsyncRead + Unpin,
    sender: broadcast::Sender<BinarySequence>,
    child: Arc<RwLock<Child>>,
) -> Result<()> {
    // let mut buf_reader = BufReader::new(output);
    loop {
        let mut buf = BytesMut::with_capacity(128);
        match output.read_buf(&mut buf).await? {
            0 => {
                // check if the process actually dead
                if ProcessState::from_child(&child).await == ProcessState::Dead {
                    break;
                }

                break;
            }
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
