use crate::app_state::AppState;
use crate::backend::http::relay_to_backend;
use crate::configuration::Configuration;
use crate::extract::UriExtract;
use crate::mailbox::http::write_to_mailbox;
use anyhow::Error;
use http_pool::body::VariantBody;
use http_pool::net_pool::Pools;
use hyper::body::Incoming;
use hyper::{Request, Response};

/// 运行
pub async fn run(app_state: AppState, conf: &Configuration) -> Result<(), Error> {
    let h = hrelay::http1::Relay::build(|b| {
        b.bind(conf.server.listen_address()).relay_fn({
            let app_state = app_state.clone();
            move |pools: Pools<http_pool::http1::Pool>, req: Request<Incoming>| {
                let app_state = app_state.clone();
                async move { relay_fn(app_state, pools, req).await }
            }
        })
    })
    .map_err(|e| Error::new(e).context("build http1 relay error"))?;

    super::run(app_state, conf, h.pools(), h);
    Ok(())
}

async fn relay_fn(
    app_state: AppState,
    pools: Pools<http_pool::http1::Pool>,
    req: Request<Incoming>,
) -> Result<Response<VariantBody>, std::io::Error> {
    let uri = req.uri().clone();
    if let Err(e) = super::check_app_state(&app_state) {
        // 暂停业务处理
        tracing::error!(
            "{}",
            format!("{:?} can't relay request cause of stopped", uri)
        );
        return Ok(e);
    }

    // 提取服务信息
    let uri = req.uri().clone();
    let extract = uri.extract_http();

    super::common_relay! {
        uri,
        app_state,
        extract,
        pools,
        req,
        write_to_mailbox,
        relay_to_backend
    }
}
