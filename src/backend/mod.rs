pub mod grpc;
pub mod http;

#[macro_export]
macro_rules! to_backend {
    (
         $pools: ident,
         $req: ident,
         $service: ident
     ) => {{
        let (pool, meta) = get_pool_and_meta(&$pools, &$service)?;

        // 获取发送者
        let mut sender = match meta.state {
            Some(ServiceState::Fixed) => {
                let service_id = $req.headers().extract_target_id();
                pool
                .clone()
                .target(get_fixed_address(&pool, service_id, meta.instances)?.get_address())
                .await
                .map_err(|e| anyhow::Error::new(e))?
            },
            _ => pool
                .get(parse_rule(&$req, &meta)?)
                .await
                .map_err(|e| anyhow::Error::new(e))?,
        };

        let uri = sender
            .new_uri($req.uri())
            .map_err(|e| anyhow::Error::new(e))?;

        *$req.uri_mut() = uri;

        sender
            .send_request($req.map(|b| variant_body(b)))
            .await
            .map(|b| b.map(|b| variant_body(b)))
            .map_err(|e| anyhow::Error::new(e))
    }};
}

pub(super) use to_backend;
