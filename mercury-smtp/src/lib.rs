use anyhow::Context as _;

pub async fn run(config: &SmtpConfig) -> anyhow::Result<()> {
    let server = smtp_server::Server::builder()
        .bind(config.addr.clone())
        .on_conn_err(|_err| {})
        .on_new_mail(|_mail| {})
        .build()
        .context("error while creating server instance")?;
    server.run().await.map_err(Into::into)
}

#[derive(serde::Deserialize)]
pub struct SmtpConfig {
    pub addr: String,
}
