use crate::inbox::data::WriteInboxRequest;
use crate::util;
use http_body_util::BodyExt;
use http_pool::body::{Full, VariantBody};
use http_pool::http2;
use http_pool::net_pool::Pools;
use hyper::body::{Bytes, Incoming};
use hyper::{http, Request, Response, StatusCode};

pub async fn write_inbox(
    writer: &kinbox::Writer,
    namespace: Option<&String>,
    pools: Pools<http2::Pool>,
    request: Request<Incoming>,
) -> Result<(Response<VariantBody>, Option<anyhow::Error>), anyhow::Error> {
    // 获取数据
    let body = request.collect().await.map_err(|e| anyhow::Error::new(e))?;

    // 解析
    let req = WriteInboxRequest::from_grpc(body.to_bytes().as_ref())
        .map_err(|e| anyhow::Error::new(e).context("parse inbox request error"))?;

    // 处理
    let (rsp, err) = super::write::write_inbox(writer, namespace, pools, req).await;

    // grpc数据流
    let buf = rsp.to_grpc()?;

    // 返回
    Ok((
        util::grpc_response(
            StatusCode::OK,
            Full::new(Bytes::from(buf)).with_trailers(async {
                let mut t = http::HeaderMap::new();
                t.insert("grpc-status", "0".parse().unwrap());
                t.insert("grpc-message", "".parse().unwrap());
                Some(Ok(t))
            }),
        ),
        err,
    ))
}
