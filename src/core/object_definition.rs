use crate::prelude::*;
use bevy::prelude::AlphaMode;

const SEPARATION_OFFSET: f32 = 0.005;

#[derive(Debug, PartialEq, Eq)]
pub enum ObjectSpecifier {
    Terrain,
    Furniture,
    Item(Item),
    Character,
    ZoneLevel,
    Meta,
}

impl ObjectSpecifier {
    pub fn vertical_offset(&self, layer: &SpriteLayer) -> f32 {
        let level = match self {
            Self::ZoneLevel => -2,
            Self::Terrain => 0,
            Self::Furniture => 2,
            Self::Item(_) => 4,
            Self::Character => 6,
            Self::Meta => return 0.15,
        } + match &layer {
            SpriteLayer::Front => 1,
            SpriteLayer::Back => 0,
        };

        level as f32 * SEPARATION_OFFSET
    }
}

#[derive(Debug)]
pub struct ObjectDefinition<'d> {
    pub name: &'d ObjectName,
    pub specifier: ObjectSpecifier,
}

impl<'d> ObjectDefinition<'d> {
    pub fn alpha_mode(&self) -> AlphaMode {
        if self.specifier == ObjectSpecifier::Terrain && self.name.is_ground() {
            AlphaMode::Opaque
        } else {
            AlphaMode::Blend
        }
    }
}
