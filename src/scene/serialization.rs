use uuid::Uuid;

const MAGIC: &[u8; 8] = b"WXRSCN\0\0";
const VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EntityData {
    pub id: Uuid,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SystemData {
    pub id: String,
    pub priority: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FieldData {
    pub name: String,
    pub value: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ComponentData {
    pub id: String,
    pub entity_id: Uuid,
    pub fields: Vec<FieldData>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SceneData {
    pub entities: Vec<EntityData>,
    pub systems: Vec<SystemData>,
    pub components: Vec<ComponentData>,
}

impl SceneData {
    pub(crate) fn encode(&self) -> Vec<u8> {
        let mut writer = Writer::new();
        writer.write_bytes(MAGIC);
        writer.write_u32(VERSION);

        writer.write_u64(self.entities.len() as u64);
        for entity in &self.entities {
            writer.write_uuid(entity.id);
            writer.write_string(&entity.name);
        }

        writer.write_u64(self.systems.len() as u64);
        for system in &self.systems {
            writer.write_string(&system.id);
            writer.write_u64(system.priority as u64);
        }

        writer.write_u64(self.components.len() as u64);
        for component in &self.components {
            writer.write_string(&component.id);
            writer.write_uuid(component.entity_id);
            writer.write_u64(component.fields.len() as u64);

            for field in &component.fields {
                writer.write_string(&field.name);
                writer.write_bytes_with_len(&field.value);
            }
        }

        writer.finish()
    }

    pub(crate) fn decode(data: &[u8]) -> Result<Self, String> {
        let mut reader = Reader::new(data);

        if reader.read_exact(MAGIC.len())? != MAGIC {
            return Err("invalid scene serialization header".to_owned());
        }

        let version = reader.read_u32()?;
        if version != VERSION {
            return Err(format!(
                "unsupported scene serialization version `{version}`"
            ));
        }

        let entity_count = reader.read_len()?;
        let mut entities = Vec::with_capacity(entity_count);
        for _ in 0..entity_count {
            entities.push(EntityData {
                id: reader.read_uuid()?,
                name: reader.read_string()?,
            });
        }

        let system_count = reader.read_len()?;
        let mut systems = Vec::with_capacity(system_count);
        for _ in 0..system_count {
            systems.push(SystemData {
                id: reader.read_string()?,
                priority: reader.read_usize()?,
            });
        }

        let component_count = reader.read_len()?;
        let mut components = Vec::with_capacity(component_count);
        for _ in 0..component_count {
            let id = reader.read_string()?;
            let entity_id = reader.read_uuid()?;
            let field_count = reader.read_len()?;
            let mut fields = Vec::with_capacity(field_count);

            for _ in 0..field_count {
                fields.push(FieldData {
                    name: reader.read_string()?,
                    value: reader.read_bytes_with_len()?,
                });
            }

            components.push(ComponentData {
                id,
                entity_id,
                fields,
            });
        }

        if !reader.is_done() {
            return Err("trailing bytes after scene serialization data".to_owned());
        }

        Ok(Self {
            entities,
            systems,
            components,
        })
    }
}

struct Writer {
    data: Vec<u8>,
}

impl Writer {
    fn new() -> Self {
        Self { data: Vec::new() }
    }

    fn finish(self) -> Vec<u8> {
        self.data
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    fn write_bytes_with_len(&mut self, bytes: &[u8]) {
        self.write_u64(bytes.len() as u64);
        self.write_bytes(bytes);
    }

    fn write_string(&mut self, value: &str) {
        self.write_bytes_with_len(value.as_bytes());
    }

    fn write_u32(&mut self, value: u32) {
        self.write_bytes(&value.to_le_bytes());
    }

    fn write_u64(&mut self, value: u64) {
        self.write_bytes(&value.to_le_bytes());
    }

    fn write_uuid(&mut self, value: Uuid) {
        self.write_bytes(value.as_bytes());
    }
}

struct Reader<'a> {
    data: &'a [u8],
    cursor: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, cursor: 0 }
    }

    fn is_done(&self) -> bool {
        self.cursor == self.data.len()
    }

    fn read_exact(&mut self, len: usize) -> Result<&'a [u8], String> {
        let end = self
            .cursor
            .checked_add(len)
            .ok_or_else(|| "serialized length overflow".to_owned())?;

        let Some(bytes) = self.data.get(self.cursor..end) else {
            return Err("unexpected end of scene serialization data".to_owned());
        };

        self.cursor = end;
        Ok(bytes)
    }

    fn read_bytes_with_len(&mut self) -> Result<Vec<u8>, String> {
        let len = self.read_len()?;
        Ok(self.read_exact(len)?.to_vec())
    }

    fn read_string(&mut self) -> Result<String, String> {
        String::from_utf8(self.read_bytes_with_len()?)
            .map_err(|_| "invalid utf-8 string in scene serialization data".to_owned())
    }

    fn read_len(&mut self) -> Result<usize, String> {
        usize::try_from(self.read_u64()?)
            .map_err(|_| "serialized length does not fit usize".to_owned())
    }

    fn read_usize(&mut self) -> Result<usize, String> {
        usize::try_from(self.read_u64()?)
            .map_err(|_| "serialized usize does not fit this platform".to_owned())
    }

    fn read_u32(&mut self) -> Result<u32, String> {
        let bytes: [u8; 4] = self
            .read_exact(4)?
            .try_into()
            .map_err(|_| "invalid u32 in scene serialization data".to_owned())?;
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_u64(&mut self) -> Result<u64, String> {
        let bytes: [u8; 8] = self
            .read_exact(8)?
            .try_into()
            .map_err(|_| "invalid u64 in scene serialization data".to_owned())?;
        Ok(u64::from_le_bytes(bytes))
    }

    fn read_uuid(&mut self) -> Result<Uuid, String> {
        let bytes: [u8; 16] = self
            .read_exact(16)?
            .try_into()
            .map_err(|_| "invalid uuid in scene serialization data".to_owned())?;
        Ok(Uuid::from_bytes(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
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

        assert_eq!(SceneData::decode(&data.encode()).unwrap(), data);
    }

    #[test]
    fn scene_data_rejects_invalid_header() {
        assert!(SceneData::decode(b"bad").is_err());
    }
}
