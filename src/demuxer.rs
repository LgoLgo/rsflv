use std::io::Write;

use crate::error::{Error, Result};
use crate::structure::{Header, HEADER_SIZE, TAG_HEADER_SIZE};
use crate::tag::{Tag, TagType};

#[derive(Debug, PartialEq, Eq)]
enum ParseState {
    Header,
    TagHeader,
    TagAndPreviousTagSize,
}

#[derive(Debug)]
pub struct Parser<F, T>
where
    F: FnMut(&mut Header) -> Result<()>,
    T: FnMut(&mut Tag) -> Result<()>,
{
    process_header: F,
    process_tag: T,

    state: ParseState,
    header_buf: Vec<u8>,
    tag_size: usize,
    tag_header_buf: Vec<u8>,
    tag_body_buf: Vec<u8>,
}

impl<F, T> Parser<F, T>
where
    F: FnMut(&mut Header) -> Result<()>,
    T: FnMut(&mut Tag) -> Result<()>,
{
    pub fn new(ph: F, pt: T) -> Self {
        Parser {
            process_header: ph,
            process_tag: pt,
            state: ParseState::Header,
            header_buf: Vec::with_capacity(HEADER_SIZE + 4),
            tag_size: 0,
            tag_header_buf: Vec::with_capacity(TAG_HEADER_SIZE),
            tag_body_buf: Vec::new(),
        }
    }

    pub fn input(&mut self, data: &[u8]) -> Result<()> {
        let total: usize = data.len();
        let mut i: usize = 0;
        let mut n: usize;
        while i < total {
            match self.state {
                ParseState::Header => {
                    n = append(&mut self.header_buf, &data[i..], HEADER_SIZE + 4);
                    // read all bytes of FLV header and previous tag size 0
                    if self.header_buf.len() != HEADER_SIZE + 4 {
                        continue;
                    }

                    // parse header
                    let mut header: Header = self.header_buf[..HEADER_SIZE].try_into()?;
                    (self.process_header)(&mut header)?;

                    // previous tag size 0
                    let previous_tag_size_0 =
                        u32::from_be_bytes(self.header_buf[HEADER_SIZE..].try_into().unwrap());
                    if previous_tag_size_0 != 0 {
                        return Err(Error::PreviousTagSize(previous_tag_size_0));
                    }
                    self.state = ParseState::TagHeader;
                }
                ParseState::TagHeader => {
                    n = append(&mut self.tag_header_buf, &data[i..], TAG_HEADER_SIZE);
                    if self.tag_header_buf.len() == TAG_HEADER_SIZE {
                        self.tag_size = u32::from_be_bytes([
                            0,
                            self.tag_header_buf[1],
                            self.tag_header_buf[2],
                            self.tag_header_buf[3],
                        ]) as usize;
                        self.state = ParseState::TagAndPreviousTagSize;
                    }
                }
                ParseState::TagAndPreviousTagSize => {
                    n = append(&mut self.tag_body_buf, &data[i..], self.tag_size + 4);
                    if self.tag_body_buf.len() == self.tag_size + 4 {
                        let mut tag = parse_tag(
                            self.tag_size,
                            &self.tag_header_buf,
                            std::mem::take(&mut self.tag_body_buf),
                        )?;
                        (self.process_tag)(&mut tag)?;
                        self.state = ParseState::TagHeader;

                        // clear
                        self.tag_header_buf.clear();
                    }
                }
            }
            i += n;
        }
        Ok(())
    }
}

// append data to buf to length n, return the number of bytes added to buf.
pub fn append(buf: &mut Vec<u8>, data: &[u8], n: usize) -> usize {
    let now = buf.len();
    if now + data.len() <= n {
        buf.write_all(data).unwrap();
        data.len()
    } else {
        buf.write_all(&data[..n - now]).unwrap();
        n - now
    }
}

pub fn parse_tag(size: usize, tag_header: &[u8], data: Vec<u8>) -> Result<Tag> {
    // verified by previousTagSizeN
    let previous_tag_size =
        u32::from_be_bytes([data[size], data[size + 1], data[size + 2], data[size + 3]]);
    if previous_tag_size as usize != size + 11 {
        return Err(Error::PreviousTagSize(previous_tag_size));
    }
    // tag header
    let tag_type = TagType::try_from(tag_header[0] & 0x1f)?;
    let timestamp =
        u32::from_be_bytes([tag_header[7], tag_header[4], tag_header[5], tag_header[6]]);
    let stream_id = u32::from_be_bytes([0, tag_header[8], tag_header[9], tag_header[10]]);

    let tag = match tag_type {
        TagType::Audio => Tag::Audio((timestamp, stream_id, data).try_into()?),
        TagType::Video => Tag::Video((timestamp, stream_id, data).try_into()?),
        TagType::Script => Tag::Script((timestamp, stream_id, data).into()),
    };
    Ok(tag)
}

#[cfg(test)]
mod test {
    use super::*;

    const TEST_FLV: &[u8] = include_bytes!("barsandtone.flv");

    #[test]
    fn test_parser() {
        let mut p = Parser::new(
            |h| {
                assert_eq!(
                    h,
                    &Header {
                        version: 1,
                        has_audio: true,
                        has_video: true,
                        data_offset: 9,
                    }
                );
                Ok(())
            },
            |tag| {
                println!("{:?}", tag);
                Ok(())
            },
        );
        p.input(TEST_FLV).unwrap();
    }
}
