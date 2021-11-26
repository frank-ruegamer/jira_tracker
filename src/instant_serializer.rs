use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use serde_with::{DeserializeAs, SerializeAs};
use std::time::{Instant, SystemTime};

pub fn serialize<S>(instant: &Instant, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let system_time = SystemTime::now() - instant.elapsed();
    system_time.serialize(serializer)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Instant, D::Error>
where
    D: Deserializer<'de>,
{
    let system_time = SystemTime::deserialize(deserializer)?;
    let duration = system_time.elapsed().map_err(Error::custom)?;
    let instant = Instant::now() - duration;
    Ok(instant)
}

pub struct SerializableInstant;

impl SerializeAs<Instant> for SerializableInstant {
    fn serialize_as<S>(source: &Instant, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize(source, serializer)
    }
}

impl<'de> DeserializeAs<'de, Instant> for SerializableInstant {
    fn deserialize_as<D>(deserializer: D) -> Result<Instant, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize(deserializer)
    }
}
