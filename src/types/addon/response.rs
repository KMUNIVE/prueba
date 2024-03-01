use derive_more::TryInto;
use serde::{de::Deserializer, Deserialize, Serialize};
use serde_with::{serde_as, VecSkipError};

use crate::types::{
    addon::DescriptorPreview,
    resource::{MetaItem, MetaItemPreview, Stream, Subtitles},
};

/// Resource Response from an addon.
///
/// Deserializing the struct from json will skip any invalid Vec items
/// and will skip any unknown to the variants fields.
#[derive(Clone, TryInto, Serialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
#[serde_as]
pub enum ResourceResponse {
    Metas {
        metas: Vec<MetaItemPreview>,
    },
    #[serde(rename_all = "camelCase")]
    MetasDetailed {
        metas_detailed: Vec<MetaItem>,
    },
    Meta {
        meta: MetaItem,
    },
    Streams {
        streams: Vec<Stream>,
    },
    Subtitles {
        subtitles: Vec<Subtitles>,
    },
    Addons {
        addons: Vec<DescriptorPreview>,
    },
}

#[serde_as]
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(transparent)]
struct SkipError<T: for<'a> Deserialize<'a> + Serialize>(
    #[serde_as(as = "VecSkipError<_>")] Vec<T>,
);

impl<'de> Deserialize<'de> for ResourceResponse {
    /// Custom deserialize for ResourceResponse which expects only 1 of the required fields
    /// to be set in the response object
    fn deserialize<D>(deserializer: D) -> Result<ResourceResponse, D::Error>
    where
        D: Deserializer<'de>,
    {
        let keys = [
            "metas",
            "metasDetailed",
            "meta",
            "streams",
            "subtitles",
            "addons",
        ];

        let value =
            serde_path_to_error::deserialize(deserializer).map_err(serde::de::Error::custom)?;
        let mut value = match value {
            serde_json::Value::Object(value) => value,
            _ => {
                return Err(serde::de::Error::custom(
                    "Cannot deserialize as ResourceResponse, expected an Object response",
                ))
            }
        };

        // check whether we have one of the expected keys or if we have more than 1 which is not allowed!
        let unique_keys_count = keys.iter().filter(|key| value.contains_key(**key)).count();
        if unique_keys_count == 0 {
            return Err(serde::de::Error::custom(
                format!("Cannot deserialize as ResourceResponse, the expected Object response didn't contain any of the required keys: {}",
                keys.join(", ")),
            ));
        }
        if unique_keys_count > 1 {
            return Err(serde::de::Error::custom(
                format!("Cannot deserialize as ResourceResponse, the expected Object response contained more than 1 of the unique keys: {}",
                keys.join(", ")),
            ));
        }

        if let Some(value) = value.get_mut("metas") {
            let skip = serde_json::from_value::<SkipError<_>>(value.take())
                .map_err(serde::de::Error::custom)?;

            Ok(ResourceResponse::Metas { metas: skip.0 })
        } else if let Some(value) = value.get_mut("metasDetailed") {
            let skip = serde_json::from_value::<SkipError<_>>(value.take())
                .map_err(serde::de::Error::custom)?;

            Ok(ResourceResponse::MetasDetailed {
                metas_detailed: skip.0,
            })
        } else if let Some(value) = value.get_mut("meta") {
            Ok(ResourceResponse::Meta {
                meta: serde_json::from_value(value.take()).map_err(serde::de::Error::custom)?,
            })
        } else if let Some(value) = value.get_mut("streams") {
            let skip = serde_json::from_value::<SkipError<_>>(value.take())
                .map_err(serde::de::Error::custom)?;

            Ok(ResourceResponse::Streams { streams: skip.0 })
        } else if let Some(value) = value.get_mut("subtitles") {
            let skip = serde_json::from_value::<SkipError<_>>(value.take())
                .map_err(serde::de::Error::custom)?;

            Ok(ResourceResponse::Subtitles { subtitles: skip.0 })
        } else if let Some(value) = value.get_mut("addons") {
            let skip = serde_json::from_value::<SkipError<_>>(value.take())
                .map_err(serde::de::Error::custom)?;

            Ok(ResourceResponse::Addons { addons: skip.0 })
        } else {
            // we should never get to this else, as we already check for missing required key
            // but we're leaving it to remove the danger of a developer forgetting to add a new key to the list.
            Err(serde::de::Error::custom(
                format!("Cannot deserialize as ResourceResponse, the expected Object response didn't contain any of the required keys: {}",
                keys.join(", ")
            )
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::from_value;

    use super::*;

    #[test]
    fn test_response_deserialization_keys() {
        // Bad json, should trigger the serde_error_to_path
        {
            // object key must be a string, number provided
            let json_response = r#"{
                "some_key": {
                    "valid_value": 9999999999999999999999999,
                    5
                }
            }"#;
            let result_err = serde_json::from_str::<ResourceResponse>(json_response)
                .expect_err("Should be an error");

            assert_eq!(
                result_err.to_string(),
                "some_key.?: key must be a string at line 4 column 21"
            );
            assert_eq!(4, result_err.line());
            assert_eq!(21, result_err.column());
        }

        // Wrong ResourceResponse, not an object response
        {
            let json_response = serde_json::json!(256);
            let result = from_value::<ResourceResponse>(json_response);

            assert!(
                result
                    .expect_err("Should be an error")
                    .to_string()
                    .contains("expected an Object response"),
                "Message does not include the text 'expected an Object response'"
            );
        }

        // Wrong ResourceResponse, missing a required key, i.e. non-existing variant
        {
            let json_response = serde_json::json!({
                "unknownVariant": {"test": 1}
            });
            let result = from_value::<ResourceResponse>(json_response);

            assert!(
                result
                    .expect_err("Should be an error")
                    .to_string()
                    .contains("didn't contain any of the required keys"),
                "Message does not include the text 'didn't contain any of the required keys'"
            );
        }

        // Wrong ResourceResponse, multiple exclusive keys, i.e. bad variant values
        {
            let json_response = serde_json::json!({
                "metas": {},
                "metasDetailed": {},
            });
            let result = from_value::<ResourceResponse>(json_response);

            assert!(
                result.expect_err("Should be an error").to_string().contains("Object response contained more than 1 of the unique keys"),
                "Message does not include the text 'Object response contained more than 1 of the unique keys'"
            );
        }
        // Wrong ResourceResponse, invalid type, expected sequence (Vec) got map (Object)
        {
            let json_response = serde_json::json!({
                "metas": {"object_key": "value"}
            });
            let result = from_value::<ResourceResponse>(json_response);

            assert!(
                result
                    .expect_err("Should be an error")
                    .to_string()
                    .contains("invalid type: map, expected a sequence"),
                "Message does not include the text 'invalid type: map, expected a sequence'"
            );
        }
    }
}
