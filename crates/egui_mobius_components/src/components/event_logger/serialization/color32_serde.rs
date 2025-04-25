use serde::{Deserialize, Deserializer, Serialize, Serializer};
use egui::Color32;

pub fn serialize<S>(color: &Color32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let rgba = [color.r(), color.g(), color.b(), color.a()];
    rgba.serialize(serializer)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Color32, D::Error>
where
    D: Deserializer<'de>,
{
    let rgba = <[u8; 4]>::deserialize(deserializer)?;
    Ok(Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3]))
}