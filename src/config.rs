use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{PosRequest, SizeEntry, SizeRequest, VideoEntry};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
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
        PosRequestConfig(SizeEntry::Percent(0.0), SizeEntry::Percent(0.0))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum SlideNumConfig {
    Single(usize),
    Many(Vec<usize>),
}

impl SlideNumConfig {
    fn as_vec(&self) -> Vec<usize> {
        match self {
            SlideNumConfig::Single(num) => vec![*num],
            SlideNumConfig::Many(vec) => vec.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct VideoConfig {
    #[serde(rename = "slide_num")]
    slide_nums: SlideNumConfig,
    video_path: PathBuf,
    #[serde(default)]
    pos: PosRequestConfig,
    size: SizeRequestConfig,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub entries: Vec<VideoConfig>,
}

impl Config {
    pub fn slides_map(self) -> HashMap<usize, Vec<VideoEntry>> {
        self.entries
            .into_iter()
            .fold(HashMap::new(), |mut acc, entry| {
                for slide_num in entry.slide_nums.as_vec() {
                    acc.entry(slide_num).or_default().push(VideoEntry {
                        slide_nums: entry.slide_nums.as_vec(),
                        video_path: entry.video_path.clone(),
                        pos: PosRequest {
                            width: entry.pos.0,
                            height: entry.pos.1,
                        },
                        size: entry.size.as_size_request(),
                    });
                }
                acc
            })
    }

    pub fn video_entries(self) -> Vec<VideoEntry> {
        self.entries
            .into_iter()
            .map(|entry| VideoEntry {
                slide_nums: entry.slide_nums.as_vec(),
                video_path: entry.video_path,
                pos: PosRequest {
                    width: entry.pos.0,
                    height: entry.pos.1,
                },
                size: entry.size.as_size_request(),
            })
            .collect()
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

        impl serde::de::Visitor<'_> for SizeEntryVisitor {
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
                    .parse::<f32>()
                    .map_err(|_e| E::invalid_value(serde::de::Unexpected::Str(value), &self))?;
                if !(0.0..=100.0).contains(&percentage) {
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
            SizeRequestConfig::Width(SizeEntry::Percent(50.0)),
            serde_json::from_str("\"50.0%\"").unwrap()
        );
        assert_eq!(
            SizeRequestConfig::WidthAndHeight(SizeEntry::Percent(50.0), SizeEntry::Percent(10.0)),
            serde_json::from_str("[\"50.0%\", \"10.0%\"]").unwrap()
        );
    }
}
