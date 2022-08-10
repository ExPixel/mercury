// SPDX-License-Identifier: GPL-3.0-or-later

use std::{error::Error, time::Duration};

use lettre::{
    message::{Attachment, Body, MultiPart, SinglePart},
    SmtpTransport, Transport,
};
use tokio::task;
use tracing::info;

#[tokio::test]
pub async fn mail_test() -> Result<(), smtp_server::Error> {
    crate::init();

    let (err_tx, mut err_rx) = tokio::sync::mpsc::channel(1);
    let (mail_tx, mut mail_rx) = tokio::sync::mpsc::channel(1);

    let server = smtp_server::Server::builder()
        .bind("localhost:8025")
        .on_conn_err(move |err| drop(err_tx.try_send(err)))
        .on_new_mail(move |mail| drop(mail_tx.try_send(mail)))
        .build()?;
    let handle = server.handle();

    let err_task = task::spawn(async move { err_rx.recv().await });
    task::spawn(async move {
        mail_rx.recv().await;
        handle.stop();
    });

    let server_task = task::spawn(async move { server.run().await });
    tokio::time::sleep(Duration::from_micros(100)).await; // wait for server to listen

    let data_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data");

    let (mailer_err_tx, mut mailer_err_rx) =
        tokio::sync::mpsc::channel::<Box<dyn Send + Sync + Error>>(1);
    task::spawn_blocking(move || {
        let img_cargo = Body::new(std::fs::read(data_dir.join("cargo.png")).expect("cargo.png"));
        let img_rust = Body::new(std::fs::read(data_dir.join("rust.svg")).expect("rust.svg"));

        let multipart = MultiPart::mixed()
            .multipart(
                MultiPart::related()
                    .singlepart(SinglePart::html(
                        "<p>Cargo: <img src=cid:cargo></p>".to_owned(),
                    ))
                    .singlepart(
                        Attachment::new_inline("cargo".to_owned())
                            .body(img_cargo, "image/png".parse().unwrap()),
                    ),
            )
            .singlepart(
                Attachment::new(String::from("rust.svg"))
                    .body(img_rust, "image/svg+xml".parse().unwrap()),
            );

        let email = lettre::Message::builder()
            .from("TestSend <test-send@example.com>".parse().unwrap())
            .to("TestRcpt <test-rcpt@example.com>".parse().unwrap())
            .subject("Test Email")
            .multipart(multipart)
            .map_err(|err| drop(mailer_err_tx.blocking_send(Box::new(err))))
            .expect("failed to build email");
        let mailer = SmtpTransport::builder_dangerous("localhost")
            .port(8025)
            .build();

        info!("sending email...");
        match mailer.send(&email) {
            Ok(_) => info!("email sent successfully"),
            Err(err) => drop(mailer_err_tx.blocking_send(Box::new(err))),
        }
        info!("sent");
    });

    let timeout = tokio::time::sleep(Duration::from_secs(100));

    tokio::select! {
        _ = timeout => panic!("timeout"),
        res = server_task => match res {
            Ok(_) => Ok(()),
            Err(err) => panic!("server error: {:?}", err),
        },
        err = mailer_err_rx.recv() => panic!("mailer error: {:?}", err),
        err = err_task => panic!("connection error: {:?}", err)
    }
}
