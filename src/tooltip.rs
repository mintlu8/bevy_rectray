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
            AnchorDirection::BL => Anchor::BOTTOM_RIGHT,
            AnchorDirection::LB => Anchor::TOP_LEFT,
            AnchorDirection::BR => Anchor::BOTTOM_LEFT,
            AnchorDirection::RB => Anchor::TOP_RIGHT,
            AnchorDirection::TL => Anchor::TOP_RIGHT,
            AnchorDirection::LT => Anchor::BOTTOM_LEFT,
            AnchorDirection::TR => Anchor::TOP_LEFT,
            AnchorDirection::RT => Anchor::BOTTOM_RIGHT,
        }
    }

    pub fn to_anchor(self) -> Anchor {
        match self {
            AnchorDirection::B => Anchor::TOP_CENTER,
            AnchorDirection::L => Anchor::CENTER_RIGHT,
            AnchorDirection::T => Anchor::BOTTOM_CENTER,
            AnchorDirection::R => Anchor::CENTER_LEFT,
            AnchorDirection::BL => Anchor::TOP_RIGHT,
            AnchorDirection::LB => Anchor::TOP_RIGHT,
            AnchorDirection::BR => Anchor::TOP_LEFT,
            AnchorDirection::RB => Anchor::TOP_LEFT,
            AnchorDirection::TL => Anchor::BOTTOM_RIGHT,
            AnchorDirection::LT => Anchor::BOTTOM_RIGHT,
            AnchorDirection::TR => Anchor::BOTTOM_LEFT,
            AnchorDirection::RT => Anchor::BOTTOM_LEFT,
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
    AnchorSwap {
        choices: [AnchorDirection; 4],
        len: u8,
    },
}

impl OutOfFrameBehavior {
    pub const fn anchor_swap(directions: &[AnchorDirection]) -> Self {
        let mut arr = [AnchorDirection::B; 4];
        let mut i = 0;
        while i < 4 && i < directions.len() {
            arr[i] = directions[i];
            i += 1;
        }
        OutOfFrameBehavior::AnchorSwap {
            choices: arr,
            len: i as u8,
        }
    }

    pub fn iter_anchor_swaps(&self) -> &[AnchorDirection] {
        match self {
            OutOfFrameBehavior::AnchorSwap { choices, len } => &choices[0..*len as usize],
            _ => &[],
        }
    }
}
