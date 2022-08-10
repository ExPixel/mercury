// SPDX-License-Identifier: GPL-3.0-or-later

mod email;

fn init() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_writer(tracing_subscriber::fmt::TestWriter::new())
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("failed to set tracing subscriber");
}
