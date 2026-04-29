use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod json_serde;
pub mod protobuf_serde;
mod to_future_record;

/// 邮箱错误码
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ErrorCode {
    /// 成功
    Ok = 0,
    /// 目标服不存在
    NoTarget = 1,
    /// 空负载
    EmptyPayload = 2,
    /// 空key
    EmptyKey = 3,
    /// 无效id
    InvalidServiceId = 4,
    /// 解包错误
    #[allow(unused)]
    ErrDecode = 5,
    /// 封包错误
    #[allow(unused)]
    ErrEncode = 6,
    /// 系统错误
    ErrSystem = 7,
    /// 目标服还未注册
    TargetNotReady = 8,
    /// 目标服没有邮箱
    NoInbox = 9,
}

/// 邮箱请求包
#[derive(Serialize, Deserialize, Debug)]
pub struct WriteInboxRequest {
    /// 目标服务
    pub service: String,
    /// key, 状态服用
    pub key: Option<String>,
    /// 服务id, 固定服用
    pub service_id: Option<u32>,
    /// 负载
    pub payload: String,
    /// 头部
    pub headers: HashMap<String, String>,
}

impl WriteInboxRequest {
    pub fn new() -> Self {
        Self {
            service: String::new(),
            key: None,
            service_id: None,
            payload: String::new(),
            headers: HashMap::new(),
        }
    }
}

/// 邮箱回得包
#[derive(Serialize, Deserialize, Debug)]
pub struct WriteInboxResponse {
    /// 与MailboxErrorCode对应
    pub code: i32,
    /// 描述信息
    pub msg: Option<String>,
}

impl WriteInboxResponse {
    pub fn new() -> Self {
        Self {
            code: ErrorCode::Ok as i32,
            msg: Some(String::from("Ok")),
        }
    }

    pub fn set_code(&mut self, code: ErrorCode) {
        match code {
            ErrorCode::Ok => {
                self.code = ErrorCode::Ok as i32;
                self.msg = Some(String::from("Ok"));
            }
            ErrorCode::NoTarget => {
                self.code = ErrorCode::NoTarget as i32;
                self.msg = Some(String::from("NoTarget"));
            }
            ErrorCode::EmptyPayload => {
                self.code = ErrorCode::EmptyPayload as i32;
                self.msg = Some(String::from("EmptyPayload"));
            }
            ErrorCode::EmptyKey => {
                self.code = ErrorCode::EmptyKey as i32;
                self.msg = Some(String::from("EmptyKey"));
            }
            ErrorCode::InvalidServiceId => {
                self.code = ErrorCode::InvalidServiceId as i32;
                self.msg = Some(String::from("InvalidServiceId"));
            }
            ErrorCode::ErrDecode => {
                self.code = ErrorCode::ErrDecode as i32;
                self.msg = Some(String::from("ErrDecode"));
            }
            ErrorCode::ErrEncode => {
                self.code = ErrorCode::ErrEncode as i32;
                self.msg = Some(String::from("ErrEncode"));
            }
            ErrorCode::ErrSystem => {
                self.code = ErrorCode::ErrSystem as i32;
                self.msg = Some(String::from("ErrSystem"));
            }
            ErrorCode::TargetNotReady => {
                self.code = ErrorCode::TargetNotReady as i32;
                self.msg = Some(String::from("TargetNotReady"));
            }
            ErrorCode::NoInbox => {
                self.code = ErrorCode::NoInbox as i32;
                self.msg = Some(String::from("NoInbox"));
            }
        }
    }
}
