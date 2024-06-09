use crate::prelude::{
    CancelHandling, ElevationVisibility, Level, Player, PlayerActionState, Pos, ZoneLevel,
};
use bevy::{
    ecs::system::SystemParam,
    prelude::{DetectChanges, Query, Ref, Res, State, States, Vec3, With},
};
use std::{cmp::Ordering, fmt};

#[derive(SystemParam)]
pub(crate) struct Focus<'w, 's> {
    pub(crate) state: Res<'w, State<FocusState>>,
    players: Query<'w, 's, Ref<'static, Pos>, With<Player>>,
}

impl<'w, 's> Focus<'w, 's> {
    pub(crate) fn is_changed(&self) -> bool {
        self.state.is_changed() || self.players.single().is_changed()
    }

    pub(crate) fn is_pos_shown(
        &self,
        shown_pos: Pos,
        elevation_visibility: ElevationVisibility,
    ) -> bool {
        let focus_pos = match **self.state {
            FocusState::Normal => *self.players.single().into_inner(),
            FocusState::ExaminingPos(pos) => pos,
            FocusState::ExaminingZoneLevel(zone_level) => {
                let focus_level = zone_level.level;
                return match (focus_level.compare_to_ground(), elevation_visibility) {
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
                };
            }
        };

        shown_pos.level <= focus_pos.level
            || (elevation_visibility == ElevationVisibility::Shown && shown_pos.z <= focus_pos.z)
    }

    pub(in crate::gameplay) fn offset(&self) -> Vec3 {
        let target_pos = match **self.state {
            FocusState::Normal => {
                return Vec3::ZERO;
            }
            FocusState::ExaminingPos(pos) => pos,
            FocusState::ExaminingZoneLevel(zone_level) => zone_level.center_pos(),
        };
        (target_pos - *self.players.single().into_inner()).vec3()
    }
}

impl<'w, 's> From<&Focus<'w, 's>> for Level {
    fn from(focus: &Focus) -> Self {
        match **focus.state {
            FocusState::Normal => focus.players.single().into_inner().level,
            FocusState::ExaminingPos(pos) => pos.level,
            FocusState::ExaminingZoneLevel(zone_level) => zone_level.level,
        }
    }
}

impl<'w, 's> From<&Focus<'w, 's>> for Pos {
    fn from(focus: &Focus) -> Self {
        match **focus.state {
            FocusState::Normal => *focus.players.single().into_inner(),
            FocusState::ExaminingPos(pos) => pos,
            FocusState::ExaminingZoneLevel(zone_level) => zone_level.center_pos(),
        }
    }
}

impl<'w, 's> From<&Focus<'w, 's>> for ZoneLevel {
    fn from(focus: &Focus) -> Self {
        match **focus.state {
            FocusState::Normal => Self::from(*focus.players.single().into_inner()),
            FocusState::ExaminingPos(pos) => Self::from(pos),
            FocusState::ExaminingZoneLevel(zone_level) => zone_level,
        }
    }
}

/// Conceptually, this is a child state of `GameplayScreenState::Base`
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
