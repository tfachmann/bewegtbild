use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{PosRequest, SizeEntry, SizeRequest, VideoEntry};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
/// TODO: Questionalable configuration.
/// Maybe explicit `width` and `height` could be preferred...
///   ```yaml
///    size:
///     width: 30%
///     height: auto
///   ```
///
enum SizeRequestConfig {
    Width(SizeEntry),
    WidthTuple((SizeEntry,)),
    WidthAndHeight(SizeEntry, SizeEntry),
}

impl SizeRequestConfig {
    fn as_size_request(self) -> SizeRequest {
        match self {
            SizeRequestConfig::Width(w) => SizeRequest::AutoHeight(w),
            SizeRequestConfig::WidthTuple((w,)) => SizeRequest::AutoHeight(w),
            SizeRequestConfig::WidthAndHeight(w, h) => SizeRequest::Size(w, h),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PosRequestConfig(SizeEntry, SizeEntry);

impl Default for PosRequestConfig {
    fn default() -> Self {
        PosRequestConfig(SizeEntry::Percent(0), SizeEntry::Percent(0))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct VideoConfig {
    slide_num: usize,
    video_path: PathBuf,
    #[serde(default)]
    pos: PosRequestConfig,
    size: SizeRequestConfig,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    entries: Vec<VideoConfig>,
}

impl Config {
    pub fn slides_map(self) -> HashMap<usize, Vec<VideoEntry>> {
        self.entries
            .into_iter()
            .fold(HashMap::new(), |mut acc, entry| {
                acc.entry(entry.slide_num).or_default().push(VideoEntry {
                    video_path: entry.video_path,
                    pos: PosRequest {
                        width: entry.pos.0,
                        height: entry.pos.1,
                    },
                    size: entry.size.as_size_request(),
                });
                acc
            })
    }
}

impl Serialize for SizeEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            SizeEntry::Percent(percent) => serializer.serialize_str(&format!("{}%", percent)),
        }
    }
}

impl<'de> Deserialize<'de> for SizeEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SizeEntryVisitor;

        impl<'de> serde::de::Visitor<'de> for SizeEntryVisitor {
            type Value = SizeEntry;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a string representing a percentage such as 20%")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let stripped = value
                    .strip_suffix('%')
                    .ok_or_else(|| E::invalid_value(serde::de::Unexpected::Str(value), &self))?;
                let percentage = stripped
                    .parse::<usize>()
                    .map_err(|_e| E::invalid_value(serde::de::Unexpected::Str(value), &self))?;
                if !(0..=100).contains(&percentage) {
                    return Err(E::invalid_value(serde::de::Unexpected::Str(value), &self));
                }
                Ok(SizeEntry::Percent(percentage))
            }
        }

        deserializer.deserialize_str(SizeEntryVisitor)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_size_request_config() {
        assert_eq!(
            SizeRequestConfig::Width(SizeEntry::Percent(50)),
            serde_json::from_str("\"50%\"").unwrap()
        );
        assert_eq!(
            SizeRequestConfig::WidthAndHeight(SizeEntry::Percent(50), SizeEntry::Percent(10)),
            serde_json::from_str("[\"50%\", \"10%\"]").unwrap()
        );
    }
}
