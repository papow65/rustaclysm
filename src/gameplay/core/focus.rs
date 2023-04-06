use crate::prelude::*;
use std::cmp::Ordering;

#[derive(PartialEq)]
pub(crate) enum Focus {
    Pos(Pos),
    ZoneLevel(ZoneLevel),
}

impl Focus {
    pub(crate) fn new(player_action_state: &PlayerActionState, player_pos: Pos) -> Self {
        match player_action_state {
            PlayerActionState::ExaminingPos(target) => Focus::Pos(*target),
            PlayerActionState::ExaminingZoneLevel(zone_level) => Focus::ZoneLevel(*zone_level),
            _ => Focus::Pos(player_pos),
        }
    }

    pub(crate) fn is_pos_shown(
        &self,
        shown_pos: Pos,
        elevation_visibility: ElevationVisibility,
    ) -> bool {
        match self {
            Focus::Pos(focus_pos) => {
                shown_pos.level <= focus_pos.level
                    || (elevation_visibility == ElevationVisibility::Shown
                        && ((shown_pos.z
                            - focus_pos.z
                            - i32::from((shown_pos.level - focus_pos.level).h))
                            < (focus_pos.x - shown_pos.x).abs()))
            }
            Focus::ZoneLevel(zone_level) => {
                let focus_level = zone_level.level;
                match (focus_level.compare_to_ground(), elevation_visibility) {
                    (Ordering::Less, _) | (Ordering::Equal, ElevationVisibility::Shown) => {
                        // Below ground elevation is ignored, so only the current level is shown.
                        // And on ground, with elevation hidden, show only the current level.
                        shown_pos.level == focus_level
                    }
                    (Ordering::Equal | Ordering::Greater, ElevationVisibility::Hidden) => {
                        // On or above ground, with elevation shown, show everything on or above ground
                        Level::ZERO <= shown_pos.level
                    }
                    (Ordering::Greater, ElevationVisibility::Shown) => {
                        // Above ground, with elevation hidden, show everything between ground and focus
                        Level::ZERO <= shown_pos.level && shown_pos.level <= focus_level
                    }
                }
            }
        }
    }
}

impl Default for Focus {
    fn default() -> Self {
        Self::Pos(Pos::ORIGIN)
    }
}

impl From<&Focus> for Level {
    fn from(focus: &Focus) -> Self {
        match focus {
            Focus::Pos(pos) => pos.level,
            Focus::ZoneLevel(zone_level) => zone_level.level,
        }
    }
}

impl From<&Focus> for Pos {
    fn from(focus: &Focus) -> Self {
        match focus {
            Focus::Pos(pos) => *pos,
            Focus::ZoneLevel(zone_level) => zone_level.base_pos(),
        }
    }
}

impl From<&Focus> for ZoneLevel {
    fn from(focus: &Focus) -> Self {
        match focus {
            Focus::Pos(pos) => ZoneLevel::from(*pos),
            Focus::ZoneLevel(zone_level) => *zone_level,
        }
    }
}
