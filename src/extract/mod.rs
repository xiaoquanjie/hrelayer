use hyper::Uri;

/// http服务提取物
#[derive(Debug, Clone)]
pub struct HttpExtract<'a> {
    pub service: &'a str,
    pub method: &'a str,
}

impl<'a> HttpExtract<'a> {
    pub fn is_mailbox(&self) -> bool {
        self.service == "inbox.inbox"
    }
}

/// grpc服务提取物
#[derive(Debug, Clone)]
pub struct GrpcExtract<'a> {
    pub package: &'a str,
    pub service: &'a str,
    pub method: &'a str,
}

impl<'a> GrpcExtract<'a> {
    pub fn is_mailbox(&self) -> bool {
        self.package == "inbox" && self.service == "inbox"
    }

    pub fn is_reflection(&self) -> bool {
        self.package == "grpc.reflection.v1"
            && self.service == "ServerReflection"
            && self.method == "ServerReflectionInfo"
    }
}

/// 提取服务名
pub trait UriExtract {
    /// http(s)://host:port/service/method
    fn extract_http(&self) -> Result<HttpExtract<'_>, &str>;

    /// http(s)://host:port/protocol.service/method
    fn extract_grpc(&self) -> Result<GrpcExtract<'_>, &str>;
}

/// 非合法的path会导致提取失败
impl UriExtract for Uri {
    fn extract_http(&self) -> Result<HttpExtract<'_>, &str> {
        let (service, method) = extract_service_method(self.path())?;
        Ok(HttpExtract { service, method })
    }

    fn extract_grpc(&self) -> Result<GrpcExtract<'_>, &str> {
        let (svc_full, method) = extract_service_method(self.path())?;

        let (package, service) = svc_full
            .rsplit_once('.')
            .filter(|(p, s)| !p.is_empty() && !s.is_empty())
            .ok_or(self.path())?;

        Ok(GrpcExtract {
            package,
            service,
            method,
        })
    }
}

fn extract_service_method(path: &str) -> Result<(&str, &str), &str> {
    Ok(path
        .trim_start_matches('/')
        .split_once('/')
        .filter(|(s, m)| !s.is_empty() && !m.is_empty())
        .ok_or(path)?)
}

/// 提取服务id
pub trait HeaderExtract {
    ///
    fn extract_target_id(&self) -> Option<u32>;

    fn extract_target(&self) -> Option<String>;

    fn is_trailer_only(&self) -> bool;

    fn is_grpc_content_type(&self) -> bool;
}

impl HeaderExtract for hyper::HeaderMap<hyper::header::HeaderValue> {
    fn extract_target_id(&self) -> Option<u32> {
        self.get("x-target-id")
            .map(|v| v.to_str().ok())
            .flatten()
            .map(|v| v.parse::<u32>().ok())
            .flatten()
    }

    fn extract_target(&self) -> Option<String> {
        self.get("x-target")
            .map(|v| v.to_str().ok())
            .flatten()
            .map(|v| v.to_string())
    }

    fn is_trailer_only(&self) -> bool {
        self.get("grpc-status").is_some() && self.get("grpc-message").is_some()
    }

    fn is_grpc_content_type(&self) -> bool {
        self.get("Content-Type")
            .map(|v| v.to_str().ok())
            .flatten()
            .filter(|v| *v == "application/grpc")
            .is_some()
    }
}
