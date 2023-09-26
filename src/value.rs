use std::collections::HashSet;

use crate::Error;

/// Common value type based on output from both node and rust
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct CommonValue {
    name: Option<String>,
    map: Option<String>,
    has_password: Option<bool>,
    players_online: Option<u64>,
    players_maximum: Option<u64>,
    player_names: HashSet<String>,
}

#[cfg(feature = "impl_rs")]
impl From<gamedig::protocols::types::CommonResponseJson<'_>> for CommonValue {
    fn from(value: gamedig::protocols::types::CommonResponseJson<'_>) -> Self {
        Self {
            name: value.name.map(|v| v.to_string()),
            map: value.map.map(|v| v.to_string()),
            has_password: value.has_password,
            players_online: Some(value.players_online.into()),
            players_maximum: Some(value.players_maximum.into()),
            player_names: value
                .players
                .map(|players| {
                    players
                        .into_iter()
                        .map(|player| player.name.to_string())
                        .collect()
                })
                .unwrap_or(HashSet::new()),
        }
    }
}

// Maybe it would be better to name these from functions
#[cfg(feature = "impl_node")]
impl TryFrom<serde_json::Value> for CommonValue {
    type Error = Error;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let obj = value
            .as_object()
            .ok_or(Error::String("Node response is not an object".to_string()))?;

        if let Some(error) = obj.get("error") {
            return Err(Error::String(error.to_string()));
        }

        Ok(Self {
            name: obj
                .get("name")
                .and_then(|v| v.as_str())
                .map(|v| v.to_string()),
            map: obj
                .get("map")
                .and_then(|v| v.as_str())
                .map(|v| v.to_string()),
            has_password: obj.get("password").and_then(|v| v.as_bool()),
            players_online: obj
                .get("players")
                .and_then(|v| v.as_array())
                .map(|v| v.len().try_into().expect("usize should fit in u64")),
            players_maximum: obj
                .get("maxplayers")
                .and_then(|v| v.as_number())
                .and_then(|v| v.as_u64()),
            player_names: obj
                .get("players")
                .and_then(|players| players.as_array())
                .map(|players| {
                    players
                        .iter()
                        .filter_map(|player| {
                            player
                                .as_object()
                                .and_then(|player| player.get("name"))
                                .and_then(|name| name.as_str())
                                .map(|name| name.to_string())
                        })
                        .collect()
                })
                .unwrap_or(HashSet::new()),
        })
    }
}
