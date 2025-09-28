use crate::encoding::Encoding;
use std::cmp::PartialEq;
use std::str::FromStr;

/// A wrapper type for content encoding that represents the compression or encoding
/// scheme used in an HTTP message body. This is used in HTTP's Content-Encoding header.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContentEncoding(Encoding);

impl ContentEncoding {
    /// Create a new ContentEncoding with the specified encoding
    pub fn new(encoding: Encoding) -> Self {
        ContentEncoding(encoding)
    }

    /// Get the encoding value
    pub fn encoding(&self) -> &Encoding {
        &self.0
    }
}

#[cfg(feature = "http_crates")]
impl headers::Header for ContentEncoding {
    fn name() -> &'static headers::HeaderName {
        &http::header::CONTENT_ENCODING
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i headers::HeaderValue>,
    {
        let mut found_encoding_optional = None;
        for header_value in values {
            let encoding = header_value
                .to_str()
                .map_err(|_| headers::Error::invalid())
                // Infallible
                .map(|e| Encoding::from_str(e).unwrap())?;

            if let Some(found_encoding) = &found_encoding_optional
                && encoding != *found_encoding
            {
                return Err(headers::Error::invalid());
            }
            // Infallible
            let _ = found_encoding_optional.insert(encoding);
        }

        match found_encoding_optional {
            None => Err(headers::Error::invalid()),
            Some(encoding) => Ok(ContentEncoding(encoding)),
        }
    }

    fn encode<E: Extend<headers::HeaderValue>>(&self, values: &mut E) {
        values.extend(headers::HeaderValue::from_str(self.0.to_string().as_str()));
    }
}

#[cfg(all(test, feature = "http_crates"))]
mod tests {
    use super::*;
    use headers::{Header, HeaderMapExt};
    use http::{HeaderMap, HeaderValue};

    #[test]
    fn test_decode_single_value() {
        let header_values = vec![HeaderValue::from_str("gzip").unwrap()];
        let content_encoding = ContentEncoding::decode(&mut header_values.iter()).unwrap();
        assert_eq!(content_encoding, ContentEncoding(Encoding::Gzip));
    }

    #[test]
    fn test_decode_multiple_identical_values() {
        let header_values = vec![
            HeaderValue::from_str("gzip").unwrap(),
            HeaderValue::from_str("gzip").unwrap(),
        ];

        let content_encoding = ContentEncoding::decode(&mut header_values.iter()).unwrap();
        assert_eq!(content_encoding, ContentEncoding(Encoding::Gzip));
    }

    #[test]
    fn test_decode_conflicting_values() {
        let header_values = vec![
            HeaderValue::from_str("gzip").unwrap(),
            HeaderValue::from_str("br").unwrap(),
        ];
        assert!(ContentEncoding::decode(&mut header_values.iter()).is_err());
    }

    #[test]
    fn test_encode() {
        let mut map = HeaderMap::new();
        let content_encoding = ContentEncoding(Encoding::Gzip);
        map.typed_insert(content_encoding);
        assert_eq!(map.get(http::header::CONTENT_ENCODING).unwrap(), "gzip");
    }
}
