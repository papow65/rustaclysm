use std::sync::LazyLock;

use crate::gameplay::{CardinalDirection, Pos, TileVariant};
use bevy::utils::HashMap;
use cdda_json_files::ObjectId;

use super::HorizontalDirection;

pub(crate) struct LocalTerrain {
    pub(crate) id: ObjectId,
    pub(crate) variant: TileVariant,
}

impl LocalTerrain {
    pub(crate) fn at(terrain: &HashMap<Pos, &ObjectId>, pos: Pos) -> Self {
        let id = terrain
            .get(&pos)
            .copied()
            .expect("Terrain id should be found");

        let similar_north = similar(terrain, pos, HorizontalDirection::North, id);
        let similar_east = similar(terrain, pos, HorizontalDirection::East, id);
        let similar_west = similar(terrain, pos, HorizontalDirection::West, id);
        let similar_south = similar(terrain, pos, HorizontalDirection::South, id);

        Self {
            id: id.clone(),
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
        }
    }

    pub(crate) const fn unconnected(id: ObjectId) -> Self {
        Self {
            id,
            variant: TileVariant::Unconnected,
        }
    }
}

fn similar(
    terrain: &HashMap<Pos, &ObjectId>,
    pos: Pos,
    direction: HorizontalDirection,
    like: &ObjectId,
) -> bool {
    static DIRT: LazyLock<ObjectId> = LazyLock::new(|| ObjectId::new("t_dirt"));
    static PAVEMENT: LazyLock<ObjectId> = LazyLock::new(|| ObjectId::new("t_pavement"));
    static PAVEMENT_DOT: LazyLock<ObjectId> = LazyLock::new(|| ObjectId::new("t_pavement_y"));

    // TODO Make the terrain for the tiles bordering this zone also available.

    let nbor = terrain.get(&pos.horizontal_nbor(direction)).copied();

    // 'nbor.is_some()' is a mostly correct hack for missing data at zone borders
    nbor == Some(like)
        || (like == &*DIRT && nbor.is_none())
        || (like == &*PAVEMENT && nbor.is_none_or(|nbor| nbor == &*PAVEMENT_DOT))
}
