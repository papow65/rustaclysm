use crate::prelude::*;
use bevy::prelude::AlphaMode;

const SEPARATION_OFFSET: f32 = 0.005;

#[derive(Debug, PartialEq)]
pub enum ObjectSpecifier {
    Terrain,
    Furniture,
    Item(Item),
    Character,
    ZoneLayer,
    Meta,
}

impl ObjectSpecifier {
    pub fn vertical_offset(&self, layer: &SpriteLayer) -> f32 {
        let level = match self {
            ObjectSpecifier::ZoneLayer => -2,
            ObjectSpecifier::Terrain => 0,
            ObjectSpecifier::Furniture => 2,
            ObjectSpecifier::Item(_) => 4,
            ObjectSpecifier::Character => 6,
            ObjectSpecifier::Meta => return 0.15,
        } + match &layer {
            SpriteLayer::Front => 1,
            SpriteLayer::Back => 0,
        };

        level as f32 * SEPARATION_OFFSET
    }
}

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
