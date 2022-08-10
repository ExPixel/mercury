// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;

use anyhow::Context as _;
use async_compression::tokio::write::GzipEncoder;
use smtp_server::RawMail;
use storage::{mail::MailMetadata, Storage};
use tokio::{io::AsyncWriteExt, sync::mpsc};
use tracing::{debug, error, warn};

pub async fn run(config: &SmtpConfig, storage: Storage) -> anyhow::Result<()> {
    let (new_mail_tx, new_mail_rx) = mpsc::unbounded_channel();
    tokio::spawn(new_mail_processing_task(new_mail_rx, storage));

    let server = smtp_server::Server::builder()
        .bind(config.address.clone())
        .on_conn_err(|err| {
            error!("connection error: {err:?}");
        })
        .on_new_mail(move |mail| {
            if let Err(_err) = new_mail_tx.send(mail) {
                warn!("received mail but mail processing task is stopped");
            }
        })
        .build()
        .context("error while creating server instance")?;
    server.run().await.map_err(Into::into)
}

async fn new_mail_processing_task(mut rx: mpsc::UnboundedReceiver<RawMail>, storage: Storage) {
    while let Some(raw_mail) = rx.recv().await {
        if let Err(err) = process_new_mail(raw_mail, &storage).await {
            error!("error while processing new mail: {err:?}");
        }
    }
}

async fn process_new_mail(raw_mail: RawMail, storage: &Storage) -> anyhow::Result<()> {
    let byte_size = raw_mail.data.len();
    tracing::debug!(bytes = byte_size, "received mail");

    let metadata = MailMetadata {
        from: raw_mail.reverse_path,
        to: raw_mail.forward_path,
    };

    let mail_id = storage
        .mail
        .store_mail_metadata(&metadata)
        .await
        .context("error occurred while storing mail metadata")?;
    debug!(id = debug(mail_id), "mail metadata stored");
    let mail_file_path = storage.mail.mail_file_path(mail_id);

    write_mail_file(&mail_file_path, &raw_mail.data)
        .await
        .with_context(|| {
            format!(
                "error occurred while writing mail data to `{}`",
                mail_file_path.display()
            )
        })?;
    debug!(path = debug(&mail_file_path), "mail data stored");

    Ok(())
}

async fn write_mail_file(path: &Path, data: &[u8]) -> anyhow::Result<()> {
    let file = tokio::fs::File::create(&path)
        .await
        .context("failed to open file")?;
    let mut encoder = GzipEncoder::new(file);
    encoder
        .write_all(data)
        .await
        .context("failed to write encoded data")?;
    encoder
        .shutdown()
        .await
        .context("failed to terminate gzip encoder")?;
    Ok(())
}

#[derive(serde::Deserialize)]
pub struct SmtpConfig {
    pub address: String,
}
