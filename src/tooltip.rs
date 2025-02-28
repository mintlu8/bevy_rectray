use bevy::{
    ecs::component::Component,
    prelude::{Reflect, ReflectComponent, ReflectDefault, ReflectDeserialize, ReflectSerialize},
};
use serde::{Deserialize, Serialize};

use crate::Anchor;
/// Anchor and direction for a tooltip like object.
///
/// * `T` means [`Anchor::TOP_CENTER`] then go up.
/// * `TL` means [`Anchor::TOP_LEFT`] then go bottom left.
/// * `LT` means [`Anchor::TOP_LEFT`] then go top right.
#[derive(Debug, Clone, Copy, Default, Reflect, Serialize, Deserialize)]
#[reflect(Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum AnchorDirection {
    #[default]
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

/// Determines how an object reacts if out of frame.
#[derive(Debug, Clone, Default, Reflect, Serialize, Deserialize, Component)]
#[reflect(Serialize, Deserialize, Component)]
pub enum OutOfFrameBehavior {
    /// Do nothing.
    #[default]
    None,
    /// If out of frame, nudge the rectangle into the frame.
    ///
    /// # Note
    ///
    /// Only works if the parent's global rotation is `0`,
    /// since we work on local transform.
    Nudge,
    /// Changes the combination of `anchor` and `parent_anchor` until in screen,
    /// if all choices failed, use `Transform2d`.
    FlexAnchor {
        choices: [AnchorDirection; 4],
        len: u8,
    },
}

impl OutOfFrameBehavior {
    pub const fn flex_anchor(directions: &[AnchorDirection]) -> Self {
        let mut arr = [AnchorDirection::B; 4];
        let mut i = 0;
        while i < 4 && i < directions.len() {
            arr[i] = directions[i];
            i += 1;
        }
        OutOfFrameBehavior::FlexAnchor {
            choices: arr,
            len: i as u8,
        }
    }

    pub fn iter_flex_anchor(&self) -> &[AnchorDirection] {
        match self {
            OutOfFrameBehavior::FlexAnchor { choices, len } => &choices[0..*len as usize],
            _ => &[],
        }
    }
}
