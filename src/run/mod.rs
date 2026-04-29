use crate::app_state::AppState;
use crate::configuration::Configuration;
use detcd::history::{History, HistoryEvent, HistoryType};
use detcd::{Service, ServiceKey};
use http_pool::body::VariantBody;
use http_pool::net_pool::{Pool, Pools};
use hyper::body::Bytes;
use hyper::{Response, StatusCode};
use net_relay::RelayExt;
use service_pool_util::pools_with_extra::PoolsWithExtra;
use service_pool_util::pools_with_service::PoolsWithService;

pub mod http1;
pub mod http2;

fn run<P: Pool + Default + 'static, H: RelayExt + 'static>(
    mut app_state: AppState,
    conf: &Configuration,
    pools: Pools<P>,
    mut h: H,
) {
    // 监控所有的服务变化
    let mut history = History::<Service, ServiceKey>::new(
        app_state.etcd_client.clone(),
        HistoryType::AllServices(conf.service.namespace.clone().unwrap()),
    );

    let name = conf.service.name.clone().unwrap_or("".to_string());
    // 启动一个协程运行
    tokio::spawn(async move {
        let remove = |s: &Service| {
            if s.key.name != name {
                pools.remove_backend_from_service(&s);
                tracing::debug!("remove service: {:?}", s);
            }
        };
        let add = |s: &Service| {
            if s.key.name != name {
                pools.write_extra_meta(&s.key.name, s.meta.as_ref().unwrap());
                pools.add_backend_from_service(&s);
                tracing::debug!("add service: {:?}", s);
            }
        };

        loop {
            tokio::select! {
                _ = h.run() => {
                },
                r = history.event() => {
                    match r {
                        Ok(e) => {
                            match e {
                                HistoryEvent::Changed(s, old) => {
                                    if let Some(old) = old {
                                        remove(&old);
                                    }
                                    add(&s);
                                },
                                HistoryEvent::Deleted(old) => {
                                    remove(&old);
                                },
                            }
                        },
                        Err(Some(es)) => {
                            for s in es {
                                remove(&s);
                            }
                        },
                        Err(_) => {}
                    }
                },
                _ = app_state.quit_rx.changed() => {
                    break;
                }
            }
        }
        tracing::info!("Relay module exit");
    });
}

fn check_app_state(app_state: &AppState) -> Result<(), Response<VariantBody>> {
    if !app_state.get_running() {
        Err(util::text_response(
            StatusCode::BAD_GATEWAY,
            Some(Bytes::from("relay stop work")),
        ))
    } else {
        Ok(())
    }
}

#[macro_export]
macro_rules! common_relay {
    (
        $uri: ident,
        $app_state: ident,
        $extract: ident,
        $pools: ident,
        $req: ident,
        $write_to_mailbox: ident,
        $relay_to_backend: ident
    ) => {
        use crate::util;
        let extract = match $extract {
            Err(_) => {
                return Ok(util::invalid_path_response());
            }
            Ok(e) => e,
        };

        if extract.is_mailbox() {
            if !extract.is_send_method() {
                return Ok(util::invalid_path_response());
            }
            let (b, e) = $write_to_mailbox(
                &$app_state.mb_builder,
                $app_state.namespace.as_ref(),
                $pools,
                $req,
            )
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{:#}", e)))?;
            if let Some(e) = e {
                tracing::error!("{:#}", e.context("mailbox operation error occurred"));
            }
            Ok(b)
        } else {
            match $relay_to_backend(extract.service.to_string(), $pools, $req).await {
                Ok(b) => Ok(b),
                Err(e) => {
                    tracing::error!("{:#}", e.context(format!("{:?} error occurred", $uri)));
                    Ok(util::invalid_path_response())
                }
            }
        }
    };
}

use crate::util;
pub use common_relay;
