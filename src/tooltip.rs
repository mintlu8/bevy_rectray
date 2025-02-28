use bevy::{
    ecs::component::Component,
    prelude::{Reflect, ReflectComponent, ReflectDefault, ReflectDeserialize, ReflectSerialize},
};
use serde::{Deserialize, Serialize};

use crate::Anchor;

/// # Meanings
///
/// `T` means [`Anchor::TOP_CENTER`] then go up.
/// `TL` means [`Anchor::TOP_LEFT`] then go bottom left.
/// `LT` means [`Anchor::TOP_LEFT`] then go top right.
#[derive(Debug, Clone, Copy, Default, Reflect, Serialize, Deserialize)]
#[reflect(Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum AnchorDirection {
    #[default]
    None,
    B,
    L,
    T,
    R,
    BL,
    LB,
    BR,
    RB,
    TL,
    LT,
    TR,
    RT,
}

impl AnchorDirection {
    pub fn to_parent_anchor(self) -> Anchor {
        match self {
            AnchorDirection::None => Anchor::CENTER,
            AnchorDirection::B => Anchor::BOTTOM_CENTER,
            AnchorDirection::L => Anchor::CENTER_LEFT,
            AnchorDirection::T => Anchor::TOP_CENTER,
            AnchorDirection::R => Anchor::CENTER_RIGHT,
            AnchorDirection::BL => todo!(),
            AnchorDirection::LB => todo!(),
            AnchorDirection::BR => todo!(),
            AnchorDirection::RB => todo!(),
            AnchorDirection::TL => todo!(),
            AnchorDirection::LT => todo!(),
            AnchorDirection::TR => todo!(),
            AnchorDirection::RT => todo!(),
        }
    }

    pub fn to_anchor(self) -> Anchor {
        match self {
            AnchorDirection::None => Anchor::CENTER,
            AnchorDirection::B => Anchor::TOP_CENTER,
            AnchorDirection::L => Anchor::CENTER_RIGHT,
            AnchorDirection::T => Anchor::BOTTOM_CENTER,
            AnchorDirection::R => Anchor::CENTER_LEFT,
            AnchorDirection::BL => todo!(),
            AnchorDirection::LB => todo!(),
            AnchorDirection::BR => todo!(),
            AnchorDirection::RB => todo!(),
            AnchorDirection::TL => todo!(),
            AnchorDirection::LT => todo!(),
            AnchorDirection::TR => todo!(),
            AnchorDirection::RT => todo!(),
        }
    }
}

/// Tries up to 4 combinations of `anchor` and `parent_anchor` combinations
/// until we find one inside the [`RectrayFrame`].
#[derive(Debug, Clone, Copy, Default, Reflect, Serialize, Deserialize, Component)]
#[reflect(Default, Serialize, Deserialize, Component)]
pub struct ToolTip([AnchorDirection; 4]);

impl ToolTip {
    pub const fn new(directions: &[AnchorDirection]) -> Self {
        let mut arr = [AnchorDirection::None; 4];
        let mut i = 0;
        while i < 4 && i < directions.len() {
            arr[i] = directions[i];
            i += 1;
        }
        ToolTip(arr)
    }
}
