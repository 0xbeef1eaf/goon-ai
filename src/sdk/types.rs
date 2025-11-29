use serde::Deserialize;
use ts_rs::TS;

#[derive(Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Deserialize, Debug, Default, Clone, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct WindowOptions {
    pub opacity: Option<f32>,
    pub position: Option<Position>,
    pub size: Option<Size>,
    pub always_on_top: Option<bool>,
    pub click_through: Option<bool>,
    pub decorations: Option<bool>,
}
