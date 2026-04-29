use anyhow::anyhow;
use detcd::{DestinationRule, Meta};
use http_pool::body::{VariantBody, variant_body};
use http_pool::net_pool::{BackendState, Pool, Pools};
use hyper::body::{Body, Buf, Bytes, Incoming};
use hyper::{Request, Response, StatusCode};
use service_pool_util::pools_with_extra::PoolsWithExtra;
use std::error::Error;
use std::sync::Arc;

/// 文本内容
pub static TEXT_CONTENT_TYPE: &str = "text/plain; charset=utf-8";
/// json内容
pub static JSON_CONTENT_TYPE: &str = "application/json";
/// grpc内容
pub static GRPC_CONTENT_TYPE: &str = "application/grpc";

pub fn get_pool_and_meta<P: Pool>(
    pools: &Pools<P>,
    service: &String,
) -> Result<(Arc<P>, Arc<Meta>), anyhow::Error> {
    // 获取连接池
    let pool = pools
        .get_pool(&service)
        .ok_or(anyhow!(format!("service/{} not exist", service)))?;

    // 获取meta
    let meta = pools
        .read_extra_meta(&service)
        .ok_or(anyhow!(format!("service/{} no meta", service)))?;

    Ok((pool, meta))
}

pub fn get_fixed_address<P: Pool>(
    pool: &Arc<P>,
    service_id: Option<u32>,
    instances: u32,
) -> Result<BackendState, anyhow::Error> {
    let service_id = service_id
        .filter(|&id| id < instances)
        .ok_or_else(|| anyhow!("invalid target service id/{:?}", service_id))?;

    let bs = pool
        .get_backend_by_id(service_id)
        .ok_or(anyhow!("target service id/{} not exist", service_id))?;

    Ok(bs)
}

pub fn parse_rule<'a>(
    req: &'a Request<Incoming>,
    meta: &'a Meta,
) -> Result<&'a str, anyhow::Error> {
    let rule = match meta.destination_rule {
        DestinationRule::Path(_) => req.uri().path(),
        DestinationRule::Header(ref h, _s) => req
            .headers()
            .get(h)
            .ok_or(anyhow!(format!("head/{} not exist", h)))?
            .to_str()
            .map_err(|_| anyhow!("head/{} not legal", h))?,
    };
    Ok(rule)
}

pub fn http_response(
    code: StatusCode,
    body: Option<Bytes>,
    content_type: &str,
) -> Response<VariantBody> {
    let builder = Response::builder()
        .status(code)
        .header("Content-Type", content_type);
    if let Some(b) = body {
        builder
            .header("Content-Length", b.len().to_string())
            .body(variant_body(http_pool::body::Full::new(b)))
            .unwrap()
    } else {
        builder.body(http_pool::body::empty()).unwrap()
    }
}

pub fn json_response(code: StatusCode, body: Option<Bytes>) -> Response<VariantBody> {
    http_response(code, body, JSON_CONTENT_TYPE)
}

pub fn text_response(code: StatusCode, body: Option<Bytes>) -> Response<VariantBody> {
    http_response(code, body, TEXT_CONTENT_TYPE)
}

pub fn grpc_response<B>(code: StatusCode, body: B) -> Response<VariantBody>
where
    B: Body + Send + Sync + 'static,
    B::Data: Buf + Send + 'static,
    B::Error: Error + Send + Sync + 'static,
{
    Response::builder()
        .status(code)
        .header("Content-Type", GRPC_CONTENT_TYPE)
        .body(variant_body(body))
        .unwrap()
}

pub fn invalid_path_response() -> Response<VariantBody> {
    text_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        Some(Bytes::from("invalid path")),
    )
}
