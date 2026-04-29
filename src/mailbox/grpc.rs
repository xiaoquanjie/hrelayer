use crate::mailbox::mailbox::WriteMailRequest;
use crate::util;
use http_body_util::BodyExt;
use http_pool::body::{Full, VariantBody};
use http_pool::http2;
use http_pool::net_pool::Pools;
use hyper::body::{Buf, Bytes, Incoming};
use hyper::{Request, Response, StatusCode, http};

pub async fn write_to_mailbox(
    builder: &kmailbox::Builder,
    namespace: Option<&String>,
    pools: Pools<http2::Pool>,
    request: Request<Incoming>,
) -> Result<(Response<VariantBody>, Option<anyhow::Error>), anyhow::Error> {
    // 获取数据
    let body = request.collect().await.map_err(|e| anyhow::Error::new(e))?;
    let mut body = body.to_bytes();
    // 去掉压缩位
    body.advance(5);

    // 解析
    let req = WriteMailRequest::from_pb(body.as_ref())
        .map_err(|e| anyhow::Error::new(e).context("parse mailbox request error"))?;

    // 处理
    let (rsp, err) = super::write_mail::write_to_mailbox(builder, namespace, pools, req).await;

    // grpc数据流
    let out = rsp.to_pb()?;
    let mut buf = Vec::with_capacity(5 + out.len());
    buf.push(0);
    buf.extend_from_slice(&(out.len() as u32).to_be_bytes());
    buf.extend_from_slice(out.as_slice());

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
