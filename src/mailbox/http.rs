use crate::mailbox::mailbox::WriteMailRequest;
use crate::util;
use http_body_util::BodyExt;
use http_pool::body::VariantBody;
use http_pool::http1;
use http_pool::net_pool::Pools;
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response, StatusCode};

pub async fn write_to_mailbox(
    builder: &kmailbox::Builder,
    namespace: Option<&String>,
    pools: Pools<http1::Pool>,
    request: Request<Incoming>,
) -> Result<(Response<VariantBody>, Option<anyhow::Error>), anyhow::Error> {
    // 获取数据
    let body = request.collect().await.map_err(|e| anyhow::Error::new(e))?;

    // 解析
    let req = WriteMailRequest::from_json(body.to_bytes().as_ref())
        .map_err(|e| anyhow::Error::new(e).context("parse mailbox request error"))?;

    // 处理
    let (rsp, err) = super::write_mail::write_to_mailbox(builder, namespace, pools, req).await;

    // 返回
    Ok((
        util::json_response(StatusCode::OK, Some(Bytes::from(rsp.to_json()?))),
        err,
    ))
}
