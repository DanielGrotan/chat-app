use bincode::{BorrowDecode, Decode, Encode, de::read::Reader, enc::write::Writer};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct Uid(pub Uuid);

impl Uid {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn inner(&self) -> Uuid {
        self.0
    }
}

impl Encode for Uid {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        encoder.writer().write(self.0.as_bytes())?;
        Ok(())
    }
}

impl<C> Decode<C> for Uid {
    fn decode<D: bincode::de::Decoder>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let mut bytes = [0u8; 16];
        decoder.reader().read(&mut bytes)?;
        Ok(Self(Uuid::from_bytes(bytes)))
    }
}

impl<'de, C> BorrowDecode<'de, C> for Uid {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = C>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let mut bytes = [0u8; 16];
        decoder.reader().read(&mut bytes)?;
        Ok(Self(Uuid::from_bytes(bytes)))
    }
}
