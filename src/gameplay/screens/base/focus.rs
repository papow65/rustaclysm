use crate::prelude::{
    CancelHandling, ElevationVisibility, Level, PlayerActionState, Pos, ZoneLevel,
};
use bevy::prelude::{States, Vec3};
use std::{cmp::Ordering, fmt};

#[derive(PartialEq)]
pub(crate) enum Focus {
    Pos(Pos),
    ZoneLevel(ZoneLevel),
}

impl Focus {
    pub(crate) const fn new(focus_state: &FocusState, player_pos: Pos) -> Self {
        match focus_state {
            FocusState::Normal => Self::Pos(player_pos),
            FocusState::ExaminingPos(target) => Self::Pos(*target),
            FocusState::ExaminingZoneLevel(zone_level) => Self::ZoneLevel(*zone_level),
        }
    }

    pub(crate) fn is_pos_shown(
        &self,
        shown_pos: Pos,
        elevation_visibility: ElevationVisibility,
    ) -> bool {
        match self {
            Self::Pos(focus_pos) => {
                shown_pos.level <= focus_pos.level
                    || (elevation_visibility == ElevationVisibility::Shown
                        && shown_pos.z <= focus_pos.z)
            }
            Self::ZoneLevel(zone_level) => {
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

    pub(in crate::gameplay) fn offset(&self, player_pos: Pos) -> Vec3 {
        match self {
            Self::Pos(target) => (*target - player_pos).vec3(),
            Self::ZoneLevel(target) => {
                (target.base_pos() - player_pos).vec3() + Vec3::new(11.5, 0.0, 11.5)
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
            Focus::Pos(pos) => Self::from(*pos),
            Focus::ZoneLevel(zone_level) => *zone_level,
        }
    }
}

/** Conceptually, this is a child state of `GameplayScreenState::Base` */
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, States)]
pub(crate) enum FocusState {
    #[default]
    Normal,
    ExaminingPos(Pos),
    ExaminingZoneLevel(ZoneLevel),
}

impl FocusState {
    pub(crate) const fn cancel_handling(
        &self,
        player_action_state: &PlayerActionState,
    ) -> CancelHandling {
        if !matches!(*self, Self::Normal) {
            CancelHandling::Queued
        } else if matches!(
            player_action_state,
            PlayerActionState::Normal | PlayerActionState::Sleeping { .. }
        ) {
            CancelHandling::Menu
        } else {
            CancelHandling::Queued
        }
    }
}

impl fmt::Display for FocusState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Normal => "",
            Self::ExaminingPos(_) => "Examining",
            Self::ExaminingZoneLevel(_) => "Examining map",
        })
    }
}
