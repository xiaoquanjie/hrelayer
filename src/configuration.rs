use anyhow::Error;
use clap::Parser;
use detcd::ServiceKey;
use serde::Deserialize;
use std::env;
use std::path::Path;
use tracing_subscriber::EnvFilter;

/// 启动参数
#[derive(Debug, Parser)]
pub struct Args {
    #[arg(short, long)]
    config: String,
}

/// 启动配置
#[derive(Debug, Clone, Deserialize)]
pub struct Configuration {
    pub server: Server,
    pub service: Service,
    pub kafka: Kafka,
    pub etcd: Etcd,
    pub log: Log,

    #[allow(dead_code)]
    pub ssl: Option<Ssl>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    pub listen: u16,
    pub protocol: String,
}

/// 注册用的服务信息
#[derive(Debug, Clone, Deserialize)]
pub struct Service {
    /// 注册的服务名
    pub name: Option<String>,
    /// 命名空间
    pub namespace: Option<String>,
    /// keepalive的有效时长
    pub ttl: i64,
    /// ip
    pub ip: String,
    /// 端口
    pub port: Option<u16>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Ssl {
    #[allow(dead_code)]
    pub use_ssl: bool,
    #[allow(dead_code)]
    pub certificate: Option<String>,
    #[allow(dead_code)]
    pub certificate_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Kafka {
    pub servers: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Etcd {
    pub endpoints: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Log {
    /// 日志等级
    pub level: u32,
    /// 日志输出路径
    pub output: String,
    /// 以下是模块日志是否开启
    pub hyper_off: bool,
    pub h2_off: bool,
    pub tonic_off: bool,
    pub tower_off: bool,
    pub etcd_off: bool,
    pub pool_off: bool,
    pub rustls_off: bool,
    pub hrelay_off: bool,
}

/// 协议
pub static GRPC_PROTOCOL: &str = "grpc";
pub static HTTP1_PROTOCOL: &str = "http";
pub static HTTP2_PROTOCOL: &str = "http2";

impl Configuration {
    pub fn build(args: &Args) -> Result<Self, Error> {
        let settings = config::Config::builder()
            .set_default("server.listen", 8080)?
            .set_default("server.protocol", HTTP1_PROTOCOL)?
            .set_default("service.ttl", 60)?
            .set_default("log.level", 1)?
            .set_default("log.output", "./")?
            .set_default("log.hyper_off", true)?
            .set_default("log.h2_off", true)?
            .set_default("log.tonic_off", true)?
            .set_default("log.tower_off", true)?
            .set_default("log.etcd_off", true)?
            .set_default("log.pool_off", true)?
            .set_default("log.rustls_off", true)?
            .set_default("log.hrelay_off", true)?
            .set_default("ssl.use_ssl", false)?
            .add_source(config::File::with_name(&args.config))
            .build()
            .map_err(|e| Error::new(e).context("read configuration file error"))?;
        settings
            .try_deserialize::<Self>()
            .map(|mut c| {
                if c.service.name.is_none() {
                    c.service.name = Some(server_name());
                }
                if c.service.port.is_none() {
                    c.service.port = Some(c.server.listen);
                }
                c
            })
            .map_err(|e| Error::new(e).context("parse configuration error"))
    }
}

impl Server {
    pub fn listen_address(&self) -> String {
        let mut a = "0.0.0.0:".to_string();
        a.push_str(&self.listen.to_string());
        a
    }

    #[allow(dead_code)]
    pub fn is_grpc(&self) -> bool {
        self.protocol == GRPC_PROTOCOL
    }

    pub fn is_http1(&self) -> bool {
        self.protocol == HTTP1_PROTOCOL
    }

    #[allow(dead_code)]
    pub fn is_http2(&self) -> bool {
        self.protocol == HTTP2_PROTOCOL
    }
}

impl Service {
    /// 注册用的service结构
    pub fn new_register_service(&self) -> detcd::Service {
        detcd::Service::from_key(ServiceKey::new(
            self.name.as_ref().unwrap(),
            self.namespace.as_ref().map_or("", |n| n),
        ))
        .ip(Some(self.ip.clone()))
        .port(self.port)
        .ttl(Some(self.ttl))
    }
}

impl Log {
    pub fn new_env_filter(&self) -> EnvFilter {
        // 日志等级
        let level = match self.level {
            0 => tracing::Level::TRACE,
            1 => tracing::Level::DEBUG,
            2 => tracing::Level::INFO,
            3 => tracing::Level::WARN,
            4 => tracing::Level::ERROR,
            _ => tracing::Level::ERROR,
        };

        let mut f = EnvFilter::new(level.as_str()).add_directive("rdkafka=off".parse().unwrap());
        if self.hyper_off {
            f = f.add_directive("hyper=off".parse().unwrap());
        }
        if self.h2_off {
            f = f.add_directive("h2=off".parse().unwrap());
        }
        if self.tonic_off {
            f = f.add_directive("tonic=off".parse().unwrap());
        }
        if self.tower_off {
            f = f.add_directive("tower=off".parse().unwrap());
        }
        if self.etcd_off {
            f = f.add_directive("etcd_detector=off".parse().unwrap());
        }
        if self.pool_off {
            f = f.add_directive("http_pool=off".parse().unwrap());
        }
        if self.rustls_off {
            f = f.add_directive("rustls=off".parse().unwrap());
        }
        if self.hrelay_off {
            f = f.add_directive("hrelay=off".parse().unwrap());
        }
        f
    }
}

/// 进程名
pub fn server_name() -> String {
    env::args()
        .next()
        .and_then(|arg0| {
            Path::new(&arg0)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
        })
        .unwrap_or("".to_string())
}
