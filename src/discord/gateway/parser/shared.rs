use std::collections::BTreeMap;

use serde_json::Value;

use crate::discord::{PresenceStatus, ids::Id};

pub(super) use crate::discord::display_name::{
    display_name_from_parts, display_name_from_parts_or_unknown,
};

pub(super) fn parse_status(value: &str) -> PresenceStatus {
    match value {
        "online" => PresenceStatus::Online,
        "idle" => PresenceStatus::Idle,
        "dnd" => PresenceStatus::DoNotDisturb,
        "offline" | "invisible" => PresenceStatus::Offline,
        _ => PresenceStatus::Unknown,
    }
}

pub(super) fn parse_id<M>(value: &Value) -> Option<Id<M>> {
    value
        .as_str()
        .and_then(|value| value.parse::<u64>().ok())
        .or_else(|| value.as_u64())
        .and_then(Id::new_checked)
}

pub(super) fn extra_fields(value: &Value, known_fields: &[&str]) -> BTreeMap<String, Value> {
    let Some(object) = value.as_object() else {
        return BTreeMap::new();
    };
    object
        .iter()
        .filter(|(field, _)| !known_fields.contains(&field.as_str()))
        .map(|(field, value)| (field.clone(), value.clone()))
        .collect()
}
