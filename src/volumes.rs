use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Volume {
    pub device: String,
    pub mountpoint: String,
    pub fs_type: String,
    #[serde(default)]
    pub options: String,
    #[serde(default)]
    pub dump: i8,
    #[serde(default)]
    pub pass: i8,
}
