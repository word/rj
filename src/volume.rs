use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Volume {
    device: String,
    mountpoint: String,
    fs_type: String,
    options: String,
    #[serde(default)]
    dump: i8,
    #[serde(default)]
    pass: i8,
}
