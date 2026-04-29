use crate::inbox::data::WriteInboxRequest;
use crate::util;
use http_body_util::BodyExt;
use http_pool::body::VariantBody;
use http_pool::http1;
use http_pool::net_pool::Pools;
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response, StatusCode};

pub async fn write_inbox(
    writer: &kinbox::Writer,
    namespace: Option<&String>,
    pools: Pools<http1::Pool>,
    request: Request<Incoming>,
) -> Result<(Response<VariantBody>, Option<anyhow::Error>), anyhow::Error> {
    // 获取数据
    let body = request.collect().await.map_err(|e| anyhow::Error::new(e))?;

    // 解析
    let req = WriteInboxRequest::from_json(body.to_bytes().as_ref())
        .map_err(|e| anyhow::Error::new(e).context("parse inbox request error"))?;

    // 处理
    let (rsp, err) = super::write::write_inbox(writer, namespace, pools, req).await;

    // 返回
    Ok((
        util::json_response(StatusCode::OK, Some(Bytes::from(rsp.to_json()?))),
        err,
    ))
}
