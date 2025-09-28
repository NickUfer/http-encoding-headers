use std::convert::Infallible;
use std::str::FromStr;

const ENC_GZIP: &str = "gzip";
const ENC_DEFLATE: &str = "deflate";
const ENC_COMPRESS: &str = "compress";
const ENC_IDENTITY: &str = "identity";
const ENC_BR: &str = "br";
const ENC_ZSTD: &str = "zstd";
const ENC_SNAPPY: &str = "snappy";
const ENC_XZ: &str = "xz";
const ENC_LZMA: &str = "lzma";
const ENC_BZIP2: &str = "bzip2";
const ENC_LZ4: &str = "lz4";
const ENC_ZLIB: &str = "zlib";
const ENC_WILDCARD: &str = "*";

pub type QualityValue = f32;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Encoding {
    Gzip,
    Deflate,
    Compress,
    Identity,
    Br,
    Zstd,
    Snappy,
    Xz,
    Lzma,
    Bzip2,
    Lz4,
    Zlib,
    Wildcard,
    Custom(String),
}

impl FromStr for Encoding {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lowercase_s = s.to_lowercase();
        match lowercase_s.as_str() {
            ENC_GZIP => Ok(Encoding::Gzip),
            ENC_DEFLATE => Ok(Encoding::Deflate),
            ENC_COMPRESS => Ok(Encoding::Compress),
            ENC_IDENTITY => Ok(Encoding::Identity),
            ENC_BR => Ok(Encoding::Br),
            ENC_ZSTD => Ok(Encoding::Zstd),
            ENC_SNAPPY => Ok(Encoding::Snappy),
            ENC_XZ => Ok(Encoding::Xz),
            ENC_LZMA => Ok(Encoding::Lzma),
            ENC_BZIP2 => Ok(Encoding::Bzip2),
            ENC_LZ4 => Ok(Encoding::Lz4),
            ENC_ZLIB => Ok(Encoding::Zlib),
            ENC_WILDCARD => Ok(Encoding::Wildcard),
            _ => Ok(Encoding::Custom(lowercase_s)),
        }
    }
}

impl std::fmt::Display for Encoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Encoding::Gzip => f.write_str(ENC_GZIP),
            Encoding::Deflate => f.write_str(ENC_DEFLATE),
            Encoding::Compress => f.write_str(ENC_COMPRESS),
            Encoding::Identity => f.write_str(ENC_IDENTITY),
            Encoding::Br => f.write_str(ENC_BR),
            Encoding::Zstd => f.write_str(ENC_ZSTD),
            Encoding::Snappy => f.write_str(ENC_SNAPPY),
            Encoding::Xz => f.write_str(ENC_XZ),
            Encoding::Lzma => f.write_str(ENC_LZMA),
            Encoding::Bzip2 => f.write_str(ENC_BZIP2),
            Encoding::Lz4 => f.write_str(ENC_LZ4),
            Encoding::Zlib => f.write_str(ENC_ZLIB),
            Encoding::Wildcard => f.write_str(ENC_WILDCARD),
            Encoding::Custom(s) => f.write_str(s),
        }
    }
}
