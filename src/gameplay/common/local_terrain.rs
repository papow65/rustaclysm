use crate::gameplay::{CardinalDirection, Pos, TileVariant, common::HorizontalDirection};
use crate::here;
use bevy::utils::HashMap;
use cdda_json_files::{RequiredLinkedLater, TerrainInfo};
use std::sync::Arc;

pub(crate) struct LocalTerrain {
    pub(crate) info: Arc<TerrainInfo>,
    pub(crate) variant: TileVariant,
}

impl LocalTerrain {
    pub(crate) fn at(
        terrain: &HashMap<Pos, &RequiredLinkedLater<TerrainInfo>>,
        pos: Pos,
    ) -> Option<Self> {
        let info = at(terrain, pos)?;

        let similar_north = similar(terrain, pos, HorizontalDirection::North, &info);
        let similar_east = similar(terrain, pos, HorizontalDirection::East, &info);
        let similar_west = similar(terrain, pos, HorizontalDirection::West, &info);
        let similar_south = similar(terrain, pos, HorizontalDirection::South, &info);

        Some(Self {
            info,
            variant: match (similar_north, similar_east, similar_west, similar_south) {
                (false, false, false, false) => TileVariant::Unconnected,
                (false, false, false, true) => TileVariant::EndPiece(CardinalDirection::South),
                (false, false, true, false) => TileVariant::EndPiece(CardinalDirection::West),
                (false, false, true, true) => TileVariant::SouthWestCorner,
                (false, true, false, false) => TileVariant::EndPiece(CardinalDirection::East),
                (false, true, false, true) => TileVariant::SouthhEastCorner,
                (false, true, true, false) => TileVariant::EastWestEdge,
                (false, true, true, true) => TileVariant::TConnection(CardinalDirection::North),
                (true, false, false, false) => TileVariant::EndPiece(CardinalDirection::North),
                (true, false, false, true) => TileVariant::NorthSouthEdge,
                (true, false, true, false) => TileVariant::NorthWestCorner,
                (true, false, true, true) => TileVariant::TConnection(CardinalDirection::East),
                (true, true, false, false) => TileVariant::NorthEastCorner,
                (true, true, false, true) => TileVariant::TConnection(CardinalDirection::West),
                (true, true, true, false) => TileVariant::TConnection(CardinalDirection::South),
                (true, true, true, true) => TileVariant::Center,
            },
        })
    }

    /// Use [`Self.at`] where possible
    pub(crate) const fn unconnected(info: Arc<TerrainInfo>) -> Self {
        Self {
            info,
            variant: TileVariant::Unconnected,
        }
    }
}

fn similar(
    terrain: &HashMap<Pos, &RequiredLinkedLater<TerrainInfo>>,
    pos: Pos,
    direction: HorizontalDirection,
    like: &Arc<TerrainInfo>,
) -> bool {
    // TODO Make the terrain for the tiles bordering this zone also available.
    at(terrain, pos.horizontal_nbor(direction)).is_none_or(|nbor| nbor.is_similar(like))
}

fn at(
    terrain: &HashMap<Pos, &RequiredLinkedLater<TerrainInfo>>,
    pos: Pos,
) -> Option<Arc<TerrainInfo>> {
    terrain.get(&pos)?.get_option(here!())
}
