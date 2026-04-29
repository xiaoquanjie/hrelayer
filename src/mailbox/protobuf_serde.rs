use super::mailbox::{WriteMailRequest, WriteMailResponse};
use quick_protobuf::sizeofs::{sizeof_len, sizeof_varint};
use quick_protobuf::{BytesReader, MessageRead, MessageWrite, Writer, WriterBackend};

impl<'a> MessageRead<'a> for WriteMailRequest {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> quick_protobuf::Result<Self> {
        let mut msg = Self::new();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.service = r.read_string(bytes)?.to_string(),
                Ok(18) => msg.key = r.read_string(bytes).ok().map(|s| s.to_string()),
                Ok(24) => msg.service_id = r.read_varint32(bytes).ok(),
                Ok(34) => msg.payload = r.read_string(bytes)?.to_string(),
                Ok(42) => {
                    let (k, v) = r.read_map(
                        bytes,
                        |r, bytes| r.read_string(bytes),
                        |r, bytes| r.read_string(bytes),
                    )?;
                    msg.headers.insert(k.to_string(), v.to_string());
                }
                Ok(t) => {
                    r.read_unknown(bytes, t)?;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

/// empty implementation
impl MessageWrite for WriteMailRequest {
    fn write_message<W: WriterBackend>(&self, _: &mut Writer<W>) -> quick_protobuf::Result<()> {
        todo!()
    }

    fn get_size(&self) -> usize {
        todo!()
    }
}

/// empty implementation
impl<'a> MessageRead<'a> for WriteMailResponse {
    fn from_reader(_: &mut BytesReader, _: &'a [u8]) -> quick_protobuf::Result<Self> {
        todo!()
    }
}

impl MessageWrite for WriteMailResponse {
    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> quick_protobuf::Result<()> {
        w.write_with_tag(8, |w| w.write_int32(self.code))?;
        if let Some(ref s) = self.msg {
            w.write_with_tag(18, |w| w.write_string(&**s))?;
        }
        Ok(())
    }

    fn get_size(&self) -> usize {
        0 + 1
            + sizeof_varint(self.code as u64)
            + self.msg.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }
}

impl WriteMailRequest {
    pub fn from_pb(data: &[u8]) -> Result<Self, quick_protobuf::Error> {
        let mut r = BytesReader::from_bytes(data);
        Self::from_reader(&mut r, data)
    }

    #[allow(unused)]
    pub fn to_pb(&self) -> Result<Vec<u8>, quick_protobuf::Error> {
        let mut buf = Vec::new();
        let mut writer = Writer::new(&mut buf);
        self.write_message(&mut writer)?;
        Ok(buf)
    }
}

impl WriteMailResponse {
    #[allow(unused)]
    pub fn from_pb(data: &[u8]) -> Result<Self, quick_protobuf::Error> {
        let mut r = BytesReader::from_bytes(data);
        Self::from_reader(&mut r, data)
    }

    pub fn to_pb(&self) -> Result<Vec<u8>, quick_protobuf::Error> {
        let mut buf = Vec::new();
        let mut writer = Writer::new(&mut buf);
        self.write_message(&mut writer)?;
        Ok(buf)
    }
}