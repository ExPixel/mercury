/*
    Mercury Mail Testing Tool
    Copyright (C) 2022 Adolph Celestin

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use anyhow::Context as _;
use config::Config;
use smtp::SmtpConfig;
use tracing::metadata::LevelFilter;
use tracing_subscriber::EnvFilter;
use web::HttpConfig;

fn main() -> anyhow::Result<()> {
    let mode = std::env::var("MERCURY_MODE").unwrap_or_else(|_| "dev".into());

    let mut config_builder = Config::builder()
        .set_default("log.level", "info")?
        .add_source(config::File::with_name("config/default"))
        .add_source(config::File::with_name(&format!("config/{}", mode)).required(false))
        .add_source(config::File::with_name("config/local").required(false))
        .add_source(config::Environment::with_prefix("MERCURY").separator("_"));

    macro_rules! config_defaults {
        ($( $name:literal = $value:expr ),* $(,)?) => {
            $(config_builder = config_builder.set_default($name, $value)?);*
        };
    }

    config_defaults! {
        "log.filter" = "info",
    };

    let config = config_builder
        .build()
        .context("error while building configuration")?;

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::ERROR.into())
                .parse(&config.get_string("log.filter")?)
                .context("error while parsing log filter")?,
        )
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("failed to set tracing subscriber");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("error while creating tokio runtime")?;

    rt.block_on(run(&config))
}

async fn run(config: &Config) -> anyhow::Result<()> {
    let http_config = config.get::<HttpConfig>("http")?;
    let smtp_config = config.get::<SmtpConfig>("smtp")?;

    let http_task = web::run(&http_config);
    let smtp_task = smtp::run(&smtp_config);

    tokio::try_join!(http_task, smtp_task).map(|(r, _)| r)
}
