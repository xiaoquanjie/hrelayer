use crate::inbox::data::{ErrorCode, WriteInboxRequest, WriteInboxResponse};
use crate::util::get_pool_and_meta;
use anyhow::anyhow;
use detcd::ServiceState;
use http_pool::net_pool::{Pool, Pools};

pub(super) async fn write_inbox<P: Pool>(
    writer: &kinbox::Writer,
    ns: Option<&String>,
    pools: Pools<P>,
    mut request: WriteInboxRequest,
) -> (WriteInboxResponse, Option<anyhow::Error>) {
    let mut response = WriteInboxResponse::new();
    let mut err = None;

    loop {
        // 校验合法性
        if let Err(c) = request.check_necessary() {
            response.set_code(c);
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

        // 判断是否有邮箱
        if !meta.with_inbox {
            response.set_code(ErrorCode::NoInbox);
            break;
        }
        
        // 获取分区
        let partition = match meta.state {
            Some(ServiceState::Fixed) => {
                // 校验service id是否合法
                if let Err(c) = request.check_service_id(meta.instances) {
                    response.set_code(c);
                    break;
                }
                request.service_id
            }
            Some(ServiceState::Stateful) => {
                // 校验key是否合法
                if let Err(c) = request.check_key() {
                    response.set_code(c);
                    break;
                }

                // 为了保证邮件的路由规则与消息包的路由规则一致, 这里要根据mb_req.key判断邮件去到哪个目标
                match pool.get_backend(request.key.as_ref().unwrap()) {
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
        let record = request.to_future_record();
        let topic = request.name(ns);
        let record = kinbox::FutureRecord {
            topic: &topic,
            ..record
        };

        match writer.write(record).await {
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
