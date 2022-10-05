// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Context as _;
use mail::header::KNOWN_HEADERS;
use smtp_server::RawMail;
use storage::Storage;
use tokio::sync::mpsc;
use tracing::{error, warn};

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

    let (_data, headers) = mail::HeaderMap::parse(&raw_mail.data)
        .map_err(|_| anyhow::Error::msg("failed to parse mail headers"))?;
    let mut known_headers = mail::HeaderMap::default();

    for header_name in KNOWN_HEADERS {
        if let Some(value) = headers.get(header_name) {
            known_headers.insert(header_name, value.to_owned());
        }
    }

    storage
        .mail
        .store_mail(&known_headers, &raw_mail.data)
        .await
        .context("error occurred while storage mail")?;

    Ok(())
}

#[derive(serde::Deserialize)]
pub struct SmtpConfig {
    pub address: String,
}
