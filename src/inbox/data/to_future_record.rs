use crate::inbox::data::{ErrorCode, WriteInboxRequest};

impl WriteInboxRequest {
    pub fn to_future_record(&self) -> kinbox::FutureRecord<'_, String, String> {
        let mut record = kinbox::FutureRecord::to("");
        record.key = self.key.as_ref();
        record.partition = self.service_id.map(|id| id as i32);
        record.payload = Some(&self.payload);

        // 填充头部
        let mut headers = kinbox::OwnedHeaders::new();
        for kv in self.headers.iter() {
            headers = headers.insert(kinbox::Header {
                key: &kv.0,
                value: Some(&kv.1),
            });
        }

        record.headers = Some(headers);
        record
    }

    pub fn name(&self, ns: Option<&String>) -> String {
        ns.map_or(self.service.clone(), |ns| {
            if ns.is_empty() {
                self.service.clone()
            } else {
                ns.clone() + "." + &self.service
            }
        })
    }

    pub fn check_necessary(&self) -> Result<(), ErrorCode> {
        if self.payload.is_empty() {
            return Err(ErrorCode::EmptyPayload);
        }
        if self.service.is_empty() {
            return Err(ErrorCode::NoTarget);
        }
        Ok(())
    }

    pub fn check_service_id(&self, instances: u32) -> Result<(), ErrorCode> {
        if self.service_id.is_none() || self.service_id.unwrap() >= instances {
            Err(ErrorCode::InvalidServiceId)
        } else {
            Ok(())
        }
    }

    pub fn check_key(&self) -> Result<(), ErrorCode> {
        if self.key.is_none() || self.key.as_ref().unwrap().is_empty() {
            Err(ErrorCode::EmptyKey)
        } else {
            Ok(())
        }
    }
}
