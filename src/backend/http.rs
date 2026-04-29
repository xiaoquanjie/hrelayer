use crate::extract::HeaderExtract;
use crate::util::{get_fixed_address, get_pool_and_meta, parse_rule};
use anyhow::Error;
use detcd::ServiceState;
use http_pool::body::{VariantBody, variant_body};
use http_pool::http1;
use http_pool::http1::HttpPool;
use http_pool::net_pool::Pools;
use hyper::body::Incoming;
use hyper::{Request, Response};

pub async fn relay_to_backend(
    service: String,
    pools: Pools<http1::Pool>,
    mut request: Request<Incoming>,
) -> Result<Response<VariantBody>, Error> {
    super::to_backend! {
        pools,
        request,
        service
    }
}
