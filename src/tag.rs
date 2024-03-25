use std::fmt::Debug;

use crate::{
    error::Error,
    structure::{
        AACPacketType, AVCPacketType, BitDepth, Channel, CodecId, FrameType, SampleRate,
        SoundFormat,
    },
};

#[derive(Debug, PartialEq, Eq)]
pub enum TagType {
    Audio = 8,
    Video = 9,
    Script = 18,
}

impl TryFrom<u8> for TagType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let tag_type: TagType = match value {
            8 => TagType::Audio,
            9 => TagType::Video,
            18 => TagType::Script,
            unknown => return Err(Error::TagType(unknown)),
        };
        Ok(tag_type)
    }
}

pub enum Tag {
    Script(Script),
    Audio(Audio),
    Video(Video),
}

impl Debug for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tag::Audio(audio) => {
                write!(
                    f,
                    "A dts:{}, stream id:{}, sound foramt:{:?}, sample rate:{:?}, bit depth:{:?}, channel:{:?}",
                    audio.timestamp,
                    audio.stream_id,
                    audio.sound_format,
                    audio.sample_rate,
                    audio.bit_depth,
                    audio.channel,
                )
            }
            Tag::Video(video) => {
                write!(
                    f,
                    "V dts:{}, pts:{}, stream_id:{}, codec:{:?}, frame type:{:?}",
                    video.dts, video.pts, video.stream_id, video.codec_id, video.frame_type
                )
            }
            Tag::Script(script) => {
                write!(
                    f,
                    "S dts:{}, stream id:{}",
                    script.timestamp, script.stream_id
                )
            }
        }
    }
}

#[derive(Debug)]
pub struct Script {
    pub timestamp: u32,
    pub stream_id: u32,

    raw_data: Vec<u8>,
}

impl Script {
    pub fn data(&self) -> &[u8] {
        &self.raw_data[..self.len()]
    }

    pub fn len(&self) -> usize {
        self.raw_data.len() - 4
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl From<(u32, u32, Vec<u8>)> for Script {
    fn from(value: (u32, u32, Vec<u8>)) -> Self {
        let (timestamp, stream_id, input) = value;
        Script {
            timestamp,
            stream_id,
            raw_data: input,
        }
    }
}

#[derive(Debug)]
pub struct Audio {
    pub timestamp: u32,
    pub stream_id: u32,

    pub sound_format: SoundFormat,
    pub sample_rate: SampleRate,
    pub bit_depth: BitDepth,
    pub channel: Channel,
    pub packet_type: Option<AACPacketType>,

    raw_data: Vec<u8>,
}

impl Audio {
    pub fn data(&self) -> &[u8] {
        if self.is_aac() {
            &self.raw_data[2..self.raw_data.len() - 4]
        } else {
            &self.raw_data[1..self.raw_data.len() - 4]
        }
    }
    pub fn len(&self) -> usize {
        self.raw_data.len() - 4
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_aac(&self) -> bool {
        self.sound_format == SoundFormat::AAC
    }
}

impl TryFrom<(u32, u32, Vec<u8>)> for Audio {
    type Error = Error;

    fn try_from(value: (u32, u32, Vec<u8>)) -> Result<Self, Self::Error> {
        let (timestamp, stream_id, input) = value;
        let sound_format = SoundFormat::try_from((input[0] >> 4) & 0x0f)?;
        let sample_rate = SampleRate::try_from((input[0] >> 2) & 0x03)?;
        let bit_depth = BitDepth::try_from((input[0] >> 1) & 0x01)?;
        let channel = Channel::try_from(input[0] & 0x01)?;
        let mut packet_type = None;
        if sound_format == SoundFormat::AAC {
            packet_type = Some(AACPacketType::try_from(input[1])?);
        }
        Ok(Audio {
            timestamp,
            stream_id,
            sound_format,
            sample_rate,
            bit_depth,
            channel,
            packet_type,
            raw_data: input,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Video {
    pub dts: u32,
    pub pts: u32,
    pub stream_id: u32,

    pub frame_type: FrameType,
    pub codec_id: CodecId,
    pub packet_type: Option<AVCPacketType>,

    raw_data: Vec<u8>,
}

impl Video {
    pub fn data(&self) -> &[u8] {
        if self.is_avc() {
            &self.raw_data[5..self.raw_data.len() - 4]
        } else {
            &self.raw_data[1..self.raw_data.len() - 4]
        }
    }

    pub fn len(&self) -> usize {
        self.raw_data.len() - 4
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_avc(&self) -> bool {
        self.codec_id == CodecId::H264
    }
}

impl TryFrom<(u32, u32, Vec<u8>)> for Video {
    type Error = Error;

    fn try_from(value: (u32, u32, Vec<u8>)) -> Result<Self, Self::Error> {
        let (timestamp, stream_id, input) = value;
        let frame_type = FrameType::try_from(input[0] >> 4 & 0xf)?;
        let codec_id = CodecId::try_from(input[0] & 0xf)?;
        let mut packet_type = None;
        let mut pts = timestamp;
        if codec_id == CodecId::H264 {
            packet_type = Some(AVCPacketType::try_from(input[1])?);
            let composition_time = u32::from_be_bytes([0, input[2], input[3], input[4]]);
            let composition_time = (composition_time + 0xff800000) ^ 0xff800000;
            pts = timestamp + composition_time;
        }
        Ok(Video {
            dts: timestamp,
            pts,
            stream_id,
            frame_type,
            codec_id,
            packet_type,
            raw_data: input,
        })
    }
}
