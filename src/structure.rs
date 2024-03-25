use crate::error::Error;

pub const SIGNATURE: &[u8] = b"FLV";
pub const HEADER_SIZE: usize = 9;
pub const TAG_HEADER_SIZE: usize = 11;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub version: u8,
    pub has_audio: bool,
    pub has_video: bool,
    pub data_offset: u32,
}

impl TryFrom<&[u8]> for Header {
    type Error = Error;

    fn try_from(input: &[u8]) -> Result<Self, Self::Error> {
        match &input[..3] {
            SIGNATURE => {}
            unknown => {
                return Err(Error::Signature(
                    unknown[0] as char,
                    unknown[1] as char,
                    unknown[2] as char,
                ));
            }
        }
        Ok(Header {
            version: input[3],
            has_audio: (input[4] & 0x04) != 0,
            has_video: (input[4] & 0x01) != 0,
            data_offset: u32::from_be_bytes([input[5], input[6], input[7], input[8]]),
        })
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundFormat {
    LinearPCM = 0,
    ADPCM = 1,
    MP3 = 2,
    PCM = 3,
    Nellymoser16KHzMono = 4,
    Nellymoser8KHzMono = 5,
    Nellymoser = 6,
    G711A = 7,
    G711U = 8,
    // reserved
    AAC = 10,
    Speex = 11,
    // reserved
    MP38KHz = 14,
    DeviceSpecificSound = 15,
}

impl TryFrom<u8> for SoundFormat {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let sound_format = match value {
            0 => SoundFormat::LinearPCM,
            1 => SoundFormat::ADPCM,
            2 => SoundFormat::MP3,
            3 => SoundFormat::PCM,
            4 => SoundFormat::Nellymoser16KHzMono,
            5 => SoundFormat::Nellymoser8KHzMono,
            6 => SoundFormat::Nellymoser,
            7 => SoundFormat::G711A,
            8 => SoundFormat::G711U,
            10 => SoundFormat::AAC,
            11 => SoundFormat::Speex,
            14 => SoundFormat::MP38KHz,
            15 => SoundFormat::DeviceSpecificSound,
            unknown => return Err(Error::InvalidSoundFormat(unknown)),
        };
        Ok(sound_format)
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SampleRate {
    _5KHz,
    _11KHz,
    _22KHz,
    _44KHz,
}

impl TryFrom<u8> for SampleRate {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let sample_rate = match value {
            0 => SampleRate::_5KHz,
            1 => SampleRate::_11KHz,
            2 => SampleRate::_22KHz,
            3 => SampleRate::_44KHz,
            unknown => return Err(Error::InvalidSampleRate(unknown)),
        };
        Ok(sample_rate)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BitDepth {
    _8Bit,
    _16Bit,
}

impl TryFrom<u8> for BitDepth {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let bit_depth = match value {
            0 => BitDepth::_8Bit,
            1 => BitDepth::_16Bit,
            unknown => return Err(Error::InvalidBitDepth(unknown)),
        };
        Ok(bit_depth)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    Mono,
    Stereo,
}

impl TryFrom<u8> for Channel {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let channel = match value {
            0 => Channel::Mono,
            1 => Channel::Stereo,
            unknown => return Err(Error::InvalidChannel(unknown)),
        };
        Ok(channel)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AACPacketType {
    SequenceHeader,
    Raw,
}

impl TryFrom<u8> for AACPacketType {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let packet_type = match value {
            0 => AACPacketType::SequenceHeader,
            1 => AACPacketType::Raw,
            unknown => return Err(Error::InvalidAACPacketType(unknown)),
        };
        Ok(packet_type)
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameType {
    KeyFrame = 1,
    InterFrame = 2,
    DisposableInterFrame = 3, // H263 only
    GeneratedKeyFrame = 4,    // reserved for server use only
    InfoFrame = 5,
}

impl TryFrom<u8> for FrameType {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let frame_type = match value {
            1 => FrameType::KeyFrame,
            2 => FrameType::InterFrame,
            3 => FrameType::DisposableInterFrame,
            4 => FrameType::GeneratedKeyFrame,
            5 => FrameType::InfoFrame,
            unknown => return Err(Error::InvalidVideoFrameType(unknown)),
        };
        Ok(frame_type)
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodecId {
    JPEG = 1,
    H263 = 2,
    ScreenVideo = 3,
    VP6 = 4,
    VP6WithAlpha = 5,
    ScreenVideo2 = 6,
    H264 = 7,
}

impl TryFrom<u8> for CodecId {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let codec_id = match value {
            1 => CodecId::JPEG,
            2 => CodecId::H263,
            3 => CodecId::ScreenVideo,
            4 => CodecId::VP6,
            5 => CodecId::VP6WithAlpha,
            6 => CodecId::ScreenVideo2,
            7 => CodecId::H264,
            unknown => return Err(Error::InvalidVideoCodecId(unknown)),
        };
        Ok(codec_id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AVCPacketType {
    SequenceHeader,
    NALUs,
    EndOfSequence,
}

impl TryFrom<u8> for AVCPacketType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let packet_type = match value {
            0 => AVCPacketType::SequenceHeader,
            1 => AVCPacketType::NALUs,
            2 => AVCPacketType::EndOfSequence,
            unknown => return Err(Error::InvalidAVCPacketType(unknown)),
        };
        Ok(packet_type)
    }
}

mod tests {
    use crate::structure::Header;

    #[test]
    fn test_header() {
        let input: &[u8] = &[b'F', b'L', b'V', 0x01, 0x05, 0x00, 0x00, 0x00, 0x09];
        assert_eq!(
            Header::try_from(input).unwrap(),
            Header {
                version: 1,
                has_audio: true,
                has_video: true,
                data_offset: 9,
            }
        );
    }

    #[test]
    fn test_signature_err() {
        let input: &[u8] = &[b'W', b'A', b'V', 0x01, 0x05, 0x00, 0x00, 0x00, 0x09];
        assert!(Header::try_from(input).is_err());
    }
}
