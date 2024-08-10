use crate::gameplay::{ElevationVisibility, FocusState, Level, Player, Pos, ZoneLevel};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{DetectChanges, NextState, Query, Ref, Res, State, Vec3, With};
use std::cmp::Ordering;

#[derive(SystemParam)]
pub(crate) struct Focus<'w, 's> {
    pub(crate) state: Res<'w, State<FocusState>>,
    players: Query<'w, 's, Ref<'static, Pos>, With<Player>>,
}

impl<'w, 's> Focus<'w, 's> {
    pub(crate) fn toggle_examine_pos(&self, next_focus_state: &mut NextState<FocusState>) {
        next_focus_state.set(match **self.state {
            FocusState::ExaminingPos(_) => FocusState::Normal,
            _ => FocusState::ExaminingPos(Pos::from(self)),
        });
    }

    pub(crate) fn toggle_examine_zone_level(&self, next_focus_state: &mut NextState<FocusState>) {
        next_focus_state.set(match **self.state {
            FocusState::ExaminingZoneLevel(_) => FocusState::Normal,
            _ => FocusState::ExaminingZoneLevel(ZoneLevel::from(self)),
        });
    }

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
