use crate::extract::HeaderExtract;
use crate::util::{get_fixed_address, get_pool_and_meta, grpc_response, parse_rule};
use anyhow::{Error, anyhow};
use detcd::ServiceState;
use http_body_util::BodyExt;
use http_pool::body::{ChannelBody, ChannelSender, VariantBody, empty, variant_body};
use http_pool::http2;
use http_pool::http2::{HttpPool, Sender};
use http_pool::net_pool::Pools;
use hyper::body::{Bytes, Frame, Incoming};
use hyper::http::request::Parts;
use hyper::{Request, Response, StatusCode, Uri};

pub async fn relay_to_backend(
    service: String,
    pools: Pools<http2::Pool>,
    mut request: Request<Incoming>,
) -> Result<Response<VariantBody>, Error> {
    let r = super::to_backend! {
        pools,
        request,
        service
    }?;

    if r.headers().is_trailer_only() {
        // 设置一个明确的空body, 防止出现多一个空data帧.且没有trailer,最后一个header帧也没有设置end_stream标识,导致
        // 有些grpc客户端无法解析出错误码
        Ok(r.map(|_| empty()))
    } else {
        Ok(r)
    }
}

pub async fn grpc_reflection(
    pools: Pools<http2::Pool>,
    request: Request<Incoming>,
) -> Result<Response<VariantBody>, Error> {
    let target = request.headers().get("x-service").map(|s| s.to_str().unwrap().to_string());
    // 拆分
    let old_uri = request.uri().clone();
    let (old_parts, old_body) = request.into_parts();

    // 创建转发单元
    let units = create_relay_unit(pools, old_uri, old_parts, target).await;
    if units.is_empty() {
        return Err(anyhow!("no grpc service"));
    }

    // 转发客户端的数据
    relay_incoming_data(old_body, units.iter().map(|u| u.tx.clone()).collect());

    // 发送请求
    let responses = relay_request(units).await;

    // 构造回包
    let (tx, new_body) = ChannelBody::new(10);

    // 转发服务器数据
    for r in responses {
        relay_incoming_data(r.into_body(), vec![tx.clone()]);
    }

    // 回包
    Ok(grpc_response(StatusCode::OK, new_body))
}

struct RelayUnit {
    tx: ChannelSender<Bytes>,
    request: Request<VariantBody>,
    sender: Sender,
}

async fn create_relay_unit(pools: Pools<http2::Pool>, uri: Uri, parts: Parts, target: Option<String>) -> Vec<RelayUnit> {
    let mut units = vec![];
    let ap = match target {
        None => pools.get_all_pools(),
        Some(t) => {
            match pools.get_pool(&t) {
                None => vec![],
                Some(p) => vec![p],
            }
        }
    };

    for pool in ap {
        if let Ok(sender) = pool.get("").await {
            let (tx, body) = ChannelBody::<Bytes>::new(10);
            let mut request = Request::from_parts(parts.clone(), variant_body(body));
            *request.uri_mut() = sender.new_uri(&uri).unwrap();
            units.push(RelayUnit {
                tx,
                request,
                sender,
            });
        }
    }
    units
}

fn relay_incoming_data(mut body: Incoming, mut txs: Vec<ChannelSender<Bytes>>) {
    tokio::spawn(async move {
        loop {
            match body.frame().await {
                None => break,
                Some(Err(_)) => break,
                Some(Ok(f)) => {
                    for tx in txs.iter_mut() {
                        if let Some(d) = f.data_ref() {
                            let _ = tx.send_frame(Frame::data(d.clone())).await;
                        }
                        if let Some(t) = f.trailers_ref() {
                            let _ = tx.send_frame(Frame::trailers(t.clone())).await;
                        }
                    }
                }
            }
        }
    });
}

async fn relay_request(units: Vec<RelayUnit>) -> Vec<Response<Incoming>> {
    let mut responses = Vec::new();
    for mut unit in units {
        match unit.sender.send_request(unit.request).await {
            Ok(r) => responses.push(r),
            Err(e) => {
                tracing::error!("relay_request error: {:?}", e);
            }
        }
    }
    responses
}


