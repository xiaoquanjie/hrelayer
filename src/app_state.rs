use crate::configuration::Configuration;
use anyhow::Error;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use tokio::sync::watch::Receiver;

/// 状态数据
#[derive(Clone)]
pub struct AppState {
    /// id
    id: u32,
    /// 命名空间
    pub namespace: Option<String>,
    /// etcd
    pub etcd_client: detcd::client::Client,
    /// inbox
    pub inbox_writer: kinbox::writer::Writer,
    /// 运行标记
    pub running: Arc<AtomicBool>,
    /// 退出
    pub quit_rx: Receiver<bool>,
}

impl AppState {
    pub async fn build(
        configuration: &Configuration,
        quit_rx: Receiver<bool>,
    ) -> Result<Self, Error> {
        Ok(Self {
            id: 0,
            namespace: configuration.service.namespace.clone(),
            etcd_client: detcd::client::Builder::new()
                .build(&configuration.etcd.endpoints)
                .await
                .map_err(|e| Error::new(e).context("connect etcd error"))?,
            inbox_writer: kinbox::writer::Writer::new_with_brokers(&configuration.kafka.servers)
                .map_err(|e| Error::new(e).context("create inbox writer error"))?,
            running: Arc::new(AtomicBool::new(true)),
            quit_rx,
        })
    }

    pub fn get_running(&self) -> bool {
        self.running.load(Relaxed)
    }

    pub fn set_running(&self, r: bool) -> &Self {
        self.running.store(r, Relaxed);
        self
    }

    pub fn set_id(&mut self, id: u32) -> &mut Self {
        self.id = id;
        self
    }

    #[allow(unused)]
    pub fn set_namespace(&mut self, ns: Option<String>) -> &mut Self {
        self.namespace = ns;
        self
    }
}
