use super::mailbox::{ErrorCode, WriteMailRequest, WriteMailResponse};
use crate::util::get_pool_and_meta;
use anyhow::anyhow;
use detcd::ServiceState;
use http_pool::net_pool::{Pool, Pools};

pub(super) async fn write_to_mailbox<P: Pool>(
    builder: &kmailbox::Builder,
    ns: Option<&String>,
    pools: Pools<P>,
    mut request: WriteMailRequest,
) -> (WriteMailResponse, Option<anyhow::Error>) {
    let mut response = WriteMailResponse::new();
    let mut err = None;

    loop {
        // 校验mb_req是否合法
        if request.payload.is_empty() {
            response.set_code(ErrorCode::EmptyPayload);
            break;
        }
        if request.service.is_empty() {
            response.set_code(ErrorCode::NoTarget);
            break;
        }

        // 获取服务类型
        let (pool, meta) = match get_pool_and_meta(&pools, &request.service) {
            Ok((p, m)) => (p, m),
            Err(e) => {
                err = Some(e);
                response.set_code(ErrorCode::NoTarget);
                break;
            }
        };

        // 获取分区
        let partition = match meta.state {
            Some(ServiceState::Fixed) => {
                // 校验service id是否合法
                if request.service_id.is_none() || request.service_id.unwrap() >= meta.instances {
                    response.set_code(ErrorCode::InvalidServiceId);
                    break;
                }
                request.service_id
            }
            Some(ServiceState::Stateful) => {
                // 校验key是否合法
                let key = request.key.as_ref().map_or("", |k| k);
                if key.is_empty() {
                    response.set_code(ErrorCode::EmptyKey);
                    break;
                }
                // 为了保证邮件的路由规则与消息包的路由规则一致, 这里要根据mb_req.key判断邮件去到哪个目标
                match pool.get_backend(key) {
                    Some(bs) => bs.id(),
                    None => {
                        response.set_code(ErrorCode::TargetNotReady);
                        break;
                    }
                }
            }
            _ => None,
        };

        // 写邮件
        request.service_id = partition;
        match write_mail(builder, ns, &request).await {
            Err(e) => {
                err = Some(anyhow!("{:?}", e));
                response.set_code(ErrorCode::ErrSystem);
                break;
            }
            _ => {}
        }

        break;
    }

    (response, err)
}

/// 发送到邮箱
async fn write_mail(
    builder: &kmailbox::Builder,
    ns: Option<&String>,
    request: &WriteMailRequest,
) -> Result<(), kmailbox::KafkaError> {
    let writer = builder.writer()?;

    // 构建邮箱结构
    let mut sm = kmailbox::SendMail::new()
        .key(request.key.as_ref())
        .payload(Some(&request.payload))
        .partition(request.service_id.map(|id| id as i32));

    // 填充头部
    for kv in request.headers.iter() {
        sm = sm.add_header(kv.0, Some(kv.1));
    }

    let name = ns.map_or(request.service.clone(), |ns| {
        if ns.is_empty() {
            request.service.clone()
        } else {
            ns.clone() + "." + &request.service
        }
    });

    writer.write(&name, sm).await
}
