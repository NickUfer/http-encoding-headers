use crate::encoding::{Encoding, QualityValue};
use std::fmt::Write;
use std::str::FromStr;
use thiserror::Error;

/// Error type for constructing `AcceptEncoding`
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum AcceptEncodingError {
    #[error("encodings cannot be empty")]
    EmptyEncodings,
}

/// Represents an HTTP Accept-Encoding header with a list of supported encodings and their quality values
#[derive(Clone)]
pub struct AcceptEncoding {
    encodings: Vec<(Encoding, QualityValue)>,
    sort: Sort,
}

/// Sort state of encodings list by quality value
#[derive(Clone)]
enum Sort {
    Ascending,
    Descending,
    Unsorted,
}

impl AcceptEncoding {
    /// Creates a new `AcceptEncoding` from a vector of encodings with their quality values.
    pub fn new(encodings: Vec<(Encoding, QualityValue)>) -> Result<Self, AcceptEncodingError> {
        if encodings.is_empty() {
            return Err(AcceptEncodingError::EmptyEncodings);
        }
        Ok(Self {
            encodings,
            sort: Sort::Unsorted,
        })
    }

    /// Returns a reference to the internal vector of encodings and their quality values.
    #[inline]
    pub fn items(&self) -> &[(Encoding, QualityValue)] {
        &self.encodings
    }

    /// Sorts the encodings by quality value in descending order and returns self.
    pub fn sort_descending(&mut self) -> &mut Self {
        self.encodings.sort_by(|a, b| b.1.total_cmp(&a.1));
        self.sort = Sort::Descending;
        self
    }

    /// Sorts the encodings by quality value in ascending order and returns self.
    pub fn sort_ascending(&mut self) -> &mut Self {
        self.encodings.sort_by(|a, b| a.1.total_cmp(&b.1));
        self.sort = Sort::Ascending;
        self
    }

    /// Returns the highest-preference encoding.
    pub fn preferred(&self) -> Option<&Encoding> {
        if self.encodings.is_empty() {
            return None;
        }
        let result = match self.sort {
            Sort::Ascending => &self.encodings[self.encodings.len() - 1].0,
            Sort::Descending => &self.encodings[0].0,
            Sort::Unsorted => self
                .encodings
                .iter()
                .max_by(|(_, weight1), (_, weight2)| weight1.total_cmp(weight2))
                .map(|(encoding, _)| encoding)
                .unwrap(),
        };
        Some(result)
    }

    /// Returns the highest-preference encoding that is also present in `allowed`.
    /// Honors current sorting state (Ascending/Descending/Unsorted) like `preferred`.
    pub fn preferred_allowed<'a>(
        &'a self,
        allowed: impl Iterator<Item = &'a Encoding>,
    ) -> Option<&'a Encoding> {
        if self.encodings.is_empty() {
            return None;
        }

        // Fast path when already sorted
        match self.sort {
            Sort::Descending => {
                use std::collections::HashSet;
                let allowed_set: HashSet<&Encoding> = allowed.collect();

                // Search from start until we find an allowed encoding
                for (enc, q) in &self.encodings {
                    if *q > 0.0 && allowed_set.contains(enc) {
                        return Some(enc);
                    }
                }
                None
            }
            Sort::Ascending => {
                use std::collections::HashSet;
                let allowed_set: HashSet<&Encoding> = allowed.collect();

                // Search from end until we find an allowed encoding
                for (enc, q) in self.encodings.iter().rev() {
                    if *q > 0.0 && allowed_set.contains(enc) {
                        return Some(enc);
                    }
                }
                None
            }
            Sort::Unsorted => {
                // Compute the max by quality among those in `allowed` and with q > 0
                let mut best: Option<(&Encoding, QualityValue)> = None;

                // To avoid repeatedly iterating allowed, collect once
                let allowed_vec = allowed.collect::<Vec<_>>();

                for (enc, q) in &self.encodings {
                    if *q <= 0.0 {
                        continue;
                    }
                    if !allowed_vec.contains(&enc) {
                        continue;
                    }
                    match best {
                        None => best = Some((enc, *q)),
                        Some((_, best_q)) if q.total_cmp(&best_q).is_gt() => best = Some((enc, *q)),
                        _ => {}
                    }
                }
                best.map(|(enc, _)| enc)
            }
        }
    }
}

#[cfg(feature = "http_crates")]
impl headers::Header for AcceptEncoding {
    fn name() -> &'static headers::HeaderName {
        &http::header::ACCEPT_ENCODING
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i headers::HeaderValue>,
    {
        let mut all_parsed: Vec<(Encoding, QualityValue)> = Vec::new();

        for header_value in values {
            let parsed = header_value
                .to_str()
                .map_err(|_| headers::Error::invalid())
                .and_then(|v| decode_header_value(v).map_err(|_| headers::Error::invalid()))?;
            all_parsed.extend(parsed);
        }

        Ok(AcceptEncoding {
            encodings: all_parsed,
            sort: Sort::Unsorted,
        })
    }

    fn encode<E: Extend<headers::HeaderValue>>(&self, values: &mut E) {
        if self.encodings.is_empty() {
            return;
        }
        let encoded = encode_header_value(&self.encodings).unwrap();
        if let Ok(hv) = headers::HeaderValue::from_str(&encoded) {
            values.extend(std::iter::once(hv));
        }
    }
}

/// Error types for Accept-Encoding header value decoding
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum AcceptEncodingDecodeError {
    #[error("encoding was empty")]
    EmptyEncodingName,
    #[error("encoding was empty")]
    EmptyEncodingWeightTuple,
    #[error("invalid quality value: {0}")]
    InvalidQualityValue(String),
    #[error("unknown directive: {0}")]
    UnexpectedDirective(String),
}

/// Decodes Accept-Encoding header value into a list of encodings with quality values
pub fn decode_header_value(
    value: &str,
) -> Result<Vec<(Encoding, QualityValue)>, AcceptEncodingDecodeError> {
    let mut parsed: Vec<(Encoding, QualityValue)> = vec![];
    for part in value.split(',') {
        let part = part.trim();
        if part.is_empty() {
            return Err(AcceptEncodingDecodeError::EmptyEncodingWeightTuple);
        }

        let mut it = part.split(';');
        let enc = it.next().map(str::trim).unwrap_or_default();
        if enc.is_empty() {
            return Err(AcceptEncodingDecodeError::EmptyEncodingName);
        }

        let mut q: QualityValue = 1.0;
        for p in it {
            let p = p.trim();
            if let Some(v) = p.strip_prefix("q=") {
                // RFC allows up to three decimals, we allow more
                q = v
                    .parse::<QualityValue>()
                    .map_err(|_| AcceptEncodingDecodeError::InvalidQualityValue(v.to_string()))?;
            } else if !p.is_empty() {
                // There is some unknown data where only a quality value
                // is expected
                return Err(AcceptEncodingDecodeError::UnexpectedDirective(
                    p.to_string(),
                ));
            }
        }

        // Infallible
        parsed.push((Encoding::from_str(enc).unwrap(), q));
    }

    Ok(parsed)
}

/// Error type for Accept-Encoding header value encoding
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum AcceptEncodingEncodeError {
    #[error("encodings cannot be empty")]
    EmptyEncodings,
}

/// Encodes a list of encodings with quality values into Accept-Encoding header value
pub fn encode_header_value(
    encodings: &[(Encoding, QualityValue)],
) -> Result<String, AcceptEncodingEncodeError> {
    if encodings.is_empty() {
        return Err(AcceptEncodingEncodeError::EmptyEncodings);
    }

    let mut buf = String::new();
    for (i, (enc, q)) in encodings.iter().enumerate() {
        if i > 0 {
            buf.push_str(", ");
        }
        buf.push_str(&enc.to_string());
        // Only include q if not exactly 1.0
        if (*q - 1.0).abs() > QualityValue::EPSILON {
            // format with up to 3 decimals, trim trailing zeros and dot
            let mut qstr = format!("{q:.3}");
            while qstr.ends_with('0') {
                qstr.pop();
            }
            if qstr.ends_with('.') {
                qstr.pop();
            }
            let _ = write!(buf, ";q={}", qstr);
        }
    }
    Ok(buf)
}

#[cfg(all(test, feature = "http_crates"))]
mod http_crates_tests {
    use super::*;
    use headers::Header;

    #[test]
    fn test_basic_decode() {
        let value = headers::HeaderValue::from_static("gzip, deflate, br");
        let mut iter = std::iter::once(&value);
        let enc = AcceptEncoding::decode(&mut iter).unwrap();

        assert_eq!(enc.items().len(), 3);
        assert!(matches!(enc.items()[0].0, Encoding::Gzip));
        assert!(matches!(enc.items()[1].0, Encoding::Deflate));
        assert!(matches!(enc.items()[2].0, Encoding::Br));
        assert!((enc.items()[0].1 - 1.0).abs() < QualityValue::EPSILON);
    }

    #[test]
    fn test_quality_values() {
        let value = headers::HeaderValue::from_static("gzip;q=1.0, deflate;q=0.5, br;q=0.1");
        let mut iter = std::iter::once(&value);
        let enc = AcceptEncoding::decode(&mut iter).unwrap();

        assert_eq!(enc.items().len(), 3);
        assert!(matches!(enc.items()[0].0, Encoding::Gzip));
        assert!((enc.items()[0].1 - 1.0).abs() < QualityValue::EPSILON);
        assert!(matches!(enc.items()[1].0, Encoding::Deflate));
        assert!((enc.items()[1].1 - 0.5).abs() < QualityValue::EPSILON);
        assert!(matches!(enc.items()[2].0, Encoding::Br));
        assert!((enc.items()[2].1 - 0.1).abs() < QualityValue::EPSILON);
    }

    #[test]
    fn test_encode() {
        let encodings = vec![
            (Encoding::Gzip, 1.0),
            (Encoding::Deflate, 0.5),
            (Encoding::Br, 0.1),
        ];
        let enc = AcceptEncoding::new(encodings).unwrap();
        let mut values = Vec::new();
        enc.encode(&mut values);

        assert_eq!(values.len(), 1);
        assert_eq!(values[0].to_str().unwrap(), "gzip, deflate;q=0.5, br;q=0.1");
    }

    #[test]
    fn test_empty() {
        let encodings = vec![];
        // constructing AcceptEncoding with empty should error
        assert!(AcceptEncoding::new(encodings).is_err());
        // and encode should not push anything when constructed with non-empty then cleared scenario isn't possible via API
    }

    #[test]
    fn test_sort_ascending() {
        let mut enc = AcceptEncoding::new(vec![
            (Encoding::Gzip, 1.0),
            (Encoding::Deflate, 0.5),
            (Encoding::Br, 0.1),
        ])
        .unwrap();
        enc.sort_ascending();

        assert_eq!(enc.items().len(), 3);
        assert!(matches!(enc.items()[0].0, Encoding::Br));
        assert!((enc.items()[0].1 - 0.1).abs() < QualityValue::EPSILON);
        assert!(matches!(enc.items()[1].0, Encoding::Deflate));
        assert!((enc.items()[1].1 - 0.5).abs() < QualityValue::EPSILON);
        assert!(matches!(enc.items()[2].0, Encoding::Gzip));
        assert!((enc.items()[2].1 - 1.0).abs() < QualityValue::EPSILON);
    }

    #[test]
    fn test_sort_descending() {
        let mut enc = AcceptEncoding::new(vec![
            (Encoding::Br, 0.1),
            (Encoding::Deflate, 0.5),
            (Encoding::Gzip, 1.0),
        ])
        .unwrap();
        enc.sort_descending();

        assert_eq!(enc.items().len(), 3);
        assert!(matches!(enc.items()[0].0, Encoding::Gzip));
        assert!((enc.items()[0].1 - 1.0).abs() < QualityValue::EPSILON);
        assert!(matches!(enc.items()[1].0, Encoding::Deflate));
        assert!((enc.items()[1].1 - 0.5).abs() < QualityValue::EPSILON);
        assert!(matches!(enc.items()[2].0, Encoding::Br));
        assert!((enc.items()[2].1 - 0.1).abs() < QualityValue::EPSILON);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_header_value_parses_list_and_qualities() {
        let parsed = decode_header_value("gzip, deflate;q=0.5, br;q=0.100").unwrap();
        assert_eq!(parsed.len(), 3);
        assert!(matches!(parsed[0].0, Encoding::Gzip));
        assert!((parsed[0].1 - 1.0).abs() < QualityValue::EPSILON);
        assert!(matches!(parsed[1].0, Encoding::Deflate));
        assert!((parsed[1].1 - 0.5).abs() < QualityValue::EPSILON);
        assert!(matches!(parsed[2].0, Encoding::Br));
        assert!((parsed[2].1 - 0.1).abs() < QualityValue::EPSILON);
    }

    #[test]
    fn decode_header_value_handles_errors() {
        // empty tuple
        assert!(matches!(
            decode_header_value(" , gzip"),
            Err(AcceptEncodingDecodeError::EmptyEncodingWeightTuple)
        ));
        // empty name
        assert!(matches!(
            decode_header_value(";q=1.0"),
            Err(AcceptEncodingDecodeError::EmptyEncodingName)
        ));
        // invalid q
        assert!(matches!(
            decode_header_value("gzip;q=abc"),
            Err(AcceptEncodingDecodeError::InvalidQualityValue(_))
        ));
        // unexpected directive
        assert!(matches!(
            decode_header_value("gzip;foo=bar"),
            Err(AcceptEncodingDecodeError::UnexpectedDirective(s)) if s=="foo=bar"
        ));
    }

    #[test]
    fn encode_header_value_formats_properly() {
        let value = encode_header_value(&[
            (Encoding::Gzip, 1.0),
            (Encoding::Deflate, 0.5),
            (Encoding::Br, 0.1),
        ])
        .unwrap();
        assert_eq!(value, "gzip, deflate;q=0.5, br;q=0.1");
    }

    #[test]
    fn encode_header_value_omits_q_for_one_and_trims_trailing_zeros() {
        let value = encode_header_value(&[
            (Encoding::Gzip, 1.0),
            (Encoding::Deflate, 0.5000),
            (Encoding::Br, 0.1000),
        ])
        .unwrap();
        // ensures trimming and omission of q=1
        assert_eq!(value, "gzip, deflate;q=0.5, br;q=0.1");
    }

    #[test]
    fn encode_header_value_errors_on_empty() {
        assert!(matches!(
            encode_header_value(&[]),
            Err(AcceptEncodingEncodeError::EmptyEncodings)
        ));
    }

    #[test]
    fn test_preferred_empty() {
        let encodings = vec![];
        let enc = AcceptEncoding::new(encodings);
        assert!(enc.is_err());
    }

    #[test]
    fn test_preferred_unsorted() {
        let enc = AcceptEncoding::new(vec![
            (Encoding::Br, 0.5),
            (Encoding::Gzip, 1.0),
            (Encoding::Deflate, 0.8),
        ])
        .unwrap();

        assert!(matches!(enc.preferred(), Some(&Encoding::Gzip)));
    }

    #[test]
    fn test_preferred_sorted_ascending() {
        let mut enc = AcceptEncoding::new(vec![
            (Encoding::Br, 0.5),
            (Encoding::Gzip, 1.0),
            (Encoding::Deflate, 0.8),
        ])
        .unwrap();
        enc.sort_ascending();

        assert!(matches!(enc.preferred(), Some(&Encoding::Gzip)));
    }

    #[test]
    fn test_preferred_sorted_descending() {
        let mut enc = AcceptEncoding::new(vec![
            (Encoding::Br, 0.5),
            (Encoding::Gzip, 1.0),
            (Encoding::Deflate, 0.8),
        ])
        .unwrap();
        enc.sort_descending();

        assert!(matches!(enc.preferred(), Some(&Encoding::Gzip)));
    }

    #[test]
    fn test_preferred_allowed_unsorted() {
        let enc = AcceptEncoding::new(vec![
            (Encoding::Br, 0.5),
            (Encoding::Gzip, 1.0),
            (Encoding::Deflate, 0.8),
        ])
        .unwrap();

        let allowed = vec![Encoding::Deflate, Encoding::Br];
        assert!(matches!(
            enc.preferred_allowed(allowed.iter()),
            Some(&Encoding::Deflate)
        ));
    }

    #[test]
    fn test_preferred_allowed_sorted_descending() {
        let mut enc = AcceptEncoding::new(vec![
            (Encoding::Br, 0.5),
            (Encoding::Gzip, 1.0),
            (Encoding::Deflate, 0.8),
        ])
        .unwrap();
        enc.sort_descending();

        let allowed = vec![Encoding::Deflate, Encoding::Br];
        assert!(matches!(
            enc.preferred_allowed(allowed.iter()),
            Some(&Encoding::Deflate)
        ));
    }

    #[test]
    fn test_preferred_allowed_sorted_ascending() {
        let mut enc = AcceptEncoding::new(vec![
            (Encoding::Br, 0.5),
            (Encoding::Gzip, 1.0),
            (Encoding::Deflate, 0.8),
        ])
        .unwrap();
        enc.sort_ascending();

        let allowed = vec![Encoding::Deflate, Encoding::Br];
        assert!(matches!(
            enc.preferred_allowed(allowed.iter()),
            Some(&Encoding::Deflate)
        ));
    }

    #[test]
    fn test_preferred_allowed_quality_zero() {
        let enc = AcceptEncoding::new(vec![
            (Encoding::Br, 0.0),
            (Encoding::Gzip, 1.0),
            (Encoding::Deflate, 0.0),
        ])
        .unwrap();

        let allowed = vec![Encoding::Deflate, Encoding::Br];
        assert!(matches!(enc.preferred_allowed(allowed.iter()), None));
    }

    #[test]
    fn test_preferred_allowed_no_matches() {
        let enc = AcceptEncoding::new(vec![
            (Encoding::Br, 0.5),
            (Encoding::Gzip, 1.0),
            (Encoding::Deflate, 0.8),
        ])
        .unwrap();

        let allowed = vec![Encoding::Identity];
        assert!(matches!(enc.preferred_allowed(allowed.iter()), None));
    }
}
