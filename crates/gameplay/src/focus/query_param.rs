use crate::{ElevationVisibility, FocusState, Level, Player, Pos, ZoneLevel};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{DetectChanges as _, NextState, Ref, Res, Single, State, Vec3, With};
use std::cmp::Ordering;

#[derive(SystemParam)]
pub(crate) struct Focus<'w> {
    state: Res<'w, State<FocusState>>,
    player_pos: Single<'w, Ref<'static, Pos>, With<Player>>,
}

impl Focus<'_> {
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
        self.state.is_changed() || self.player_pos.is_changed()
    }

    pub(crate) fn is_pos_shown(
        &self,
        shown_pos: Pos,
        elevation_visibility: ElevationVisibility,
    ) -> bool {
        let focus_pos = match **self.state {
            FocusState::Normal => **self.player_pos,
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

    pub(crate) fn offset(&self) -> Vec3 {
        let target_pos = match **self.state {
            FocusState::Normal => {
                return Vec3::ZERO;
            }
            FocusState::ExaminingPos(pos) => pos,
            FocusState::ExaminingZoneLevel(zone_level) => zone_level.center_pos(),
        };
        (target_pos - **self.player_pos).vec3()
    }
}

impl From<&Focus<'_>> for Level {
    fn from(focus: &Focus) -> Self {
        match **focus.state {
            FocusState::Normal => focus.player_pos.level,
            FocusState::ExaminingPos(pos) => pos.level,
            FocusState::ExaminingZoneLevel(zone_level) => zone_level.level,
        }
    }
}

impl From<&Focus<'_>> for Pos {
    fn from(focus: &Focus) -> Self {
        match **focus.state {
            FocusState::Normal => **focus.player_pos,
            FocusState::ExaminingPos(pos) => pos,
            FocusState::ExaminingZoneLevel(zone_level) => zone_level.center_pos(),
        }
    }
}

impl From<&Focus<'_>> for ZoneLevel {
    fn from(focus: &Focus) -> Self {
        match **focus.state {
            FocusState::Normal => Self::from(**focus.player_pos),
            FocusState::ExaminingPos(pos) => Self::from(pos),
            FocusState::ExaminingZoneLevel(zone_level) => zone_level,
        }
    }
}
