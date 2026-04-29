use crate::app_state::AppState;
use crate::configuration::{Args, Configuration};
use anyhow::Error;
use clap::Parser;
use detcd::Service;
use detcd::registrar::Registrar;
use run::{http1, http2};
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::sleep;

mod app_state;
mod backend;
mod configuration;
mod extract;
mod inbox;
mod run;
mod trace;
mod util;

#[tokio::main]
async fn main() {
    let configuration = match Configuration::build(&Args::parse()) {
        Ok(c) => c,
        Err(e) => {
            println!("{:#}", e);
            return;
        }
    };

    println!("{:?}", configuration);

    let to_console;
    let to_file;

    #[cfg(debug_assertions)]
    {
        to_console = true;
        to_file = false;
    }

    #[cfg(not(debug_assertions))]
    {
        to_console = false;
        to_file = true;
    }

    // 初始化日志
    let _guard = trace::init(&configuration, to_file, to_console);

    if let Err(e) = run(configuration).await {
        tracing::error!("{:#}", e);
    }

    tracing::info!("server exit!!!!");
}

async fn run(configuration: Configuration) -> Result<(), Error> {
    let (quit_tx, quit_rx) = watch::channel(false);

    // 创建app数据
    let mut app_state = AppState::build(&configuration, quit_rx).await?;

    // 注册服务自己
    let mut registrar = Registrar::<Service>::from((
        app_state.etcd_client.clone(),
        configuration.service.new_register_service(),
    ));

    registrar
        .register()
        .await
        .map_err(|e| Error::new(e).context("register service error"))?;

    // 设置状态
    app_state
        .set_id(registrar.service().key.id.unwrap())
        .set_running(registrar.status().registered());

    if configuration.server.is_http1() {
        http1::run(app_state.clone(), &configuration).await?;
    } else {
        http2::run(app_state.clone(), &configuration).await?;
    }

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                let _ = quit_tx.send(true);
                tracing::info!("waiting to exit gracefully");
                break;
            }
            status = registrar.changed() => {
                app_state.set_running(status.registered());
            }
        }
    }

    drop(registrar);

    // wait for seconds
    sleep(Duration::from_secs(5)).await;
    tracing::info!("exit ok");
    Ok(())
}
