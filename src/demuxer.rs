use std::io::Read;

use crate::error::{Error, Result};
use crate::structure::{Header, HEADER_SIZE};

pub struct Demuxer<R> {
    reader: R,
}

impl<R: Read> Demuxer<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn read_header(&mut self) -> Result<Header> {
        let mut buf = [0u8; HEADER_SIZE];
        self.reader.read_exact(&mut buf)?; // io error
    
        let header: Header = Header::try_from(buf.as_slice())?; // signature error
    
        // previousTagSize0
        let (buf, _) = buf.split_at_mut(4);
        self.reader.read_exact(buf)?; // io error
        let previous_tag_size_0 = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        if previous_tag_size_0 != 0 {
            return Err(Error::PreviousTagSize(previous_tag_size_0.into()));
        }
    
        Ok(header)
    }
}

#[cfg(test)]
mod tests {
    use super::Demuxer;
    use crate::error::Result;
    use crate::{error::Error, structure::Header};
    use std::io::BufReader;
    use test_case::test_case;

    #[test_case(&[b'F', b'L', b'V'] => matches Err(Error::IO(..)) ; "only signature")]
    #[test_case(&[b'F', b'L', b'V', 0x01, 0x05, 0x00, 0x00, 0x00, 0x09] => matches Err(Error::IO(..)) ; "only header")]
    #[test_case(&[b'F', b'L', b'V', 0x01, 0x05, 0x00, 0x00, 0x00, 0x09,0x0,0x0,0x0,0x1] => matches Err(Error::PreviousTagSize(..)) ; "invalid previous tag size0")]
    #[test_case(&[
        b'F', b'L', b'V', 0x01, 0x05, 0x00, 0x00, 0x00, 0x09, 0x0, 0x0, 0x0, 0x0,
    ] => matches Ok(Header {
        version: 1,
        has_audio: true, 
        has_video: true,
        data_offset: 9,
    }) ; "Ok")]
    fn test_read_header(input: &[u8]) -> Result<Header> {
        let reader = BufReader::new(input);
        let mut demuxer = Demuxer::new(reader);
        demuxer.read_header()
    }
}