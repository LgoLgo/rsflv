use std::io::Read;

pub struct Demuxer<R> {
    reader: R,
}

impl<R: Read> Demuxer<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }
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
        return Err(Error::PreviousTagSize(previous_tag_size_0));
    }

    Ok(header)
}