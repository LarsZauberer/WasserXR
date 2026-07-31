use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MAGIC: &[u8; 8] = b"WXRSCN\0\0";
const VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct EntityData {
    pub id: Uuid,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct SystemData {
    pub id: String,
    pub priority: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct FieldData {
    pub name: String,
    pub value: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct ComponentData {
    pub id: String,
    pub entity_id: Uuid,
    pub fields: Vec<FieldData>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct SceneData {
    pub entities: Vec<EntityData>,
    pub systems: Vec<SystemData>,
    pub components: Vec<ComponentData>,
}

impl SceneData {
    pub(crate) fn encode(&self) -> Result<Vec<u8>, String> {
        let mut data = Vec::new();
        data.extend_from_slice(MAGIC);
        data.extend_from_slice(&VERSION.to_le_bytes());

        let mut payload = bincode::serde::encode_to_vec(self, bincode::config::standard())
            .map_err(|error| error.to_string())?;
        data.append(&mut payload);

        Ok(data)
    }

    pub(crate) fn decode(data: &[u8]) -> Result<Self, String> {
        let Some(header) = data.get(..MAGIC.len()) else {
            return Err("invalid scene serialization header".to_owned());
        };

        if header != MAGIC {
            return Err("invalid scene serialization header".to_owned());
        }

        let version_start = MAGIC.len();
        let version_end = version_start + std::mem::size_of::<u32>();
        let version_bytes: [u8; 4] = data
            .get(version_start..version_end)
            .ok_or_else(|| "missing scene serialization version".to_owned())?
            .try_into()
            .map_err(|_| "invalid scene serialization version".to_owned())?;

        let version = u32::from_le_bytes(version_bytes);
        if version != VERSION {
            return Err(format!(
                "unsupported scene serialization version `{version}`"
            ));
        }

        let payload = data
            .get(version_end..)
            .ok_or_else(|| "missing scene serialization payload".to_owned())?;

        let (scene, bytes_read): (Self, usize) =
            bincode::serde::decode_from_slice(payload, bincode::config::standard())
                .map_err(|error| error.to_string())?;

        if bytes_read != payload.len() {
            return Err("trailing bytes after scene serialization data".to_owned());
        }

        Ok(scene)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    fn scene_data_round_trip() {
        let entity_id = Uuid::now_v7();
        let data = SceneData {
            entities: vec![EntityData {
                id: entity_id,
                name: "Player".to_owned(),
            }],
            systems: vec![SystemData {
                id: "movement".to_owned(),
                priority: 2,
            }],
            components: vec![ComponentData {
                id: "transform".to_owned(),
                entity_id,
                fields: vec![FieldData {
                    name: "x".to_owned(),
                    value: vec![1, 2, 3],
                }],
            }],
        };

        assert_eq!(SceneData::decode(&data.encode().unwrap()).unwrap(), data);
    }

    #[rstest]
    fn scene_data_rejects_invalid_header() {
        assert!(SceneData::decode(b"bad").is_err());
    }
}
