use crate::prelude::{
    Accessible, Clock, Envir, Level, LevelOffset, Player, PlayerActionState, Pos, PosOffset,
    RelativeSegment, RelativeSegments, SubzoneLevel, Visible, VisionDistance,
};
use bevy::{
    ecs::system::SystemParam,
    prelude::{Query, Res, State, With},
    utils::HashMap,
};
use std::cell::RefCell;

#[derive(SystemParam)]
pub(crate) struct CurrentlyVisibleBuilder<'w, 's> {
    pub(crate) envir: Envir<'w, 's>,
    relative_segments: Res<'w, RelativeSegments>,
    clock: Clock<'w>,
    player_action_state: Res<'w, State<PlayerActionState>>,
    players: Query<'w, 's, &'static Pos, With<Player>>,
}

impl<'w, 's> CurrentlyVisibleBuilder<'w, 's> {
    pub(crate) fn for_npc(&self, pos: Pos) -> CurrentlyVisible {
        let viewing_distance = CurrentlyVisible::viewing_distance(&self.clock, None, pos.level);
        self.build(viewing_distance, pos, true)
    }

    pub(crate) fn for_player(&self, only_nearby: bool) -> CurrentlyVisible {
        let from_pos = if let PlayerActionState::Peeking {
            active_target: Some(target),
        } = **self.player_action_state
        {
            self.player_pos().horizontal_nbor(target.into())
        } else {
            self.player_pos()
        };
        let viewing_distance = CurrentlyVisible::viewing_distance(
            &self.clock,
            Some(&*self.player_action_state),
            from_pos.level,
        );
        self.build(viewing_distance, from_pos, only_nearby)
    }

    fn build(
        &self,
        viewing_distance: Option<usize>,
        from: Pos,
        only_nearby: bool,
    ) -> CurrentlyVisible {
        // segments are not used when viewing_distance is None, so then we pick any.
        let segments = self
            .relative_segments
            .segments
            .get(viewing_distance.unwrap_or(0))
            .unwrap_or_else(|| panic!("{viewing_distance:?}"));

        let magic_stairs_up = self
            .envir
            .magic_stairs_up()
            .map(|pos| pos - from)
            .collect::<Vec<_>>();
        let magic_stairs_down = self
            .envir
            .magic_stairs_down()
            .map(|pos| pos - from)
            .collect::<Vec<_>>();

        let visible_cache = RefCell::<HashMap<PosOffset, Visible>>::default();
        if magic_stairs_up.contains(&PosOffset::HERE) {
            if let Some(up) = self.envir.stairs_up_to(from) {
                visible_cache.borrow_mut().insert(up - from, Visible::Seen);
            }
        } else if magic_stairs_down.contains(&PosOffset::HERE) {
            if let Some(down) = self.envir.stairs_down_to(from) {
                visible_cache
                    .borrow_mut()
                    .insert(down - from, Visible::Seen);
            }
        }

        let nearby_subzone_level_cache = only_nearby.then(RefCell::default);

        CurrentlyVisible {
            envir: &self.envir,
            segments,
            viewing_distance,
            from,
            opaque_cache: RefCell::default(),
            down_cache: RefCell::default(),
            visible_cache,
            nearby_subzone_level_cache,
            magic_stairs_up,
            magic_stairs_down,
        }
    }

    pub(crate) fn player_pos(&self) -> Pos {
        *self.players.single()
    }
}

pub(crate) struct CurrentlyVisible<'a> {
    envir: &'a Envir<'a, 'a>,
    segments: &'a HashMap<PosOffset, RelativeSegment>,

    /// Rounded up in calculations - None when not even 'from' is visible
    viewing_distance: Option<usize>,
    from: Pos,
    opaque_cache: RefCell<HashMap<PosOffset, bool>>, // is opaque
    down_cache: RefCell<HashMap<PosOffset, bool>>,   // can see down
    visible_cache: RefCell<HashMap<PosOffset, Visible>>,

    /// None is used when everything should be updated
    nearby_subzone_level_cache: Option<RefCell<HashMap<SubzoneLevel, bool>>>,

    /// The stairs up that do not have stairs down directly above them
    magic_stairs_up: Vec<PosOffset>,
    /// The stairs down that do not have stairs up directly below them
    magic_stairs_down: Vec<PosOffset>,
}

impl<'a> CurrentlyVisible<'a> {
    const MIN_DISTANCE: f32 = 3.0;

    pub(crate) fn viewing_distance(
        clock: &Clock,
        player_action_state: Option<&PlayerActionState>,
        level: Level,
    ) -> Option<usize> {
        if let Some(PlayerActionState::Sleeping { .. }) = player_action_state {
            None
        } else {
            let light = if level < Level::ZERO {
                0.0
            } else {
                clock.sunlight_percentage()
            };
            Some(
                (light * VisionDistance::MAX_VISION_TILES as f32
                    + (1.0 - light) * Self::MIN_DISTANCE) as usize,
            )
        }
    }

    pub(crate) fn can_see(&self, to: Pos, accessible: Option<&Accessible>) -> Visible {
        // We ignore floors seen from below. Those are not particulary interesting and require complex logic to do properly.

        if self.nearby_pos(to, 0) && (to.level <= self.from.level || accessible.is_none()) {
            self.can_see_relative(to - self.from)
        } else {
            Visible::Unseen
        }
    }

    const fn nearby_pos(&self, pos: Pos, extra: usize) -> bool {
        let Some(viewing_distance) = self.viewing_distance else {
            return false;
        };

        self.from.x.abs_diff(pos.x) as usize <= viewing_distance + extra
            && self.from.z.abs_diff(pos.z) as usize <= viewing_distance + extra
    }

    pub(crate) fn can_see_relative(&self, relative_to: PosOffset) -> Visible {
        if let Some(visible) = self.visible_cache.borrow().get(&relative_to) {
            return visible.clone();
        }

        let Some(relative_segment) = self.segments.get(&relative_to) else {
            return self.remember_visible(relative_to, Visible::Unseen);
        };

        if relative_segment
            .preceding
            .map(|preceding| self.can_see_relative(preceding))
            == Some(Visible::Unseen)
        {
            return self.remember_visible(relative_to, Visible::Unseen);
        }

        if relative_segment
            .segment
            .iter()
            .any(|offset| self.is_opaque(*offset))
        {
            return self.remember_visible(relative_to, Visible::Unseen);
        }

        let visible = if relative_segment
            .down_pairs
            .iter()
            .all(|(offset_a, offset_b)| {
                self.can_look_vertical(*offset_a) || self.can_look_vertical(*offset_b)
            }) {
            Visible::Seen
        } else {
            Visible::Unseen
        };
        self.remember_visible(relative_to, visible)
    }

    fn is_opaque(&self, offset: PosOffset) -> bool {
        *self
            .opaque_cache
            .borrow_mut()
            .entry(offset)
            .or_insert_with(|| {
                let to = self.from.offset(offset).expect("Valid offset");
                self.envir.is_opaque(to)
                    || (to.level < Level::ZERO && self.envir.find_terrain(to).is_none())
            })
    }

    fn can_look_vertical(&self, above: PosOffset) -> bool {
        *self
            .down_cache
            .borrow_mut()
            .entry(above)
            .or_insert_with(|| {
                if LevelOffset::ZERO < above.level {
                    // looking up
                    let below = above.down();
                    if self.magic_stairs_up.contains(&below) {
                        return false;
                    }
                } else if self.magic_stairs_down.contains(&above) {
                    return false;
                }

                let above = self.from.offset(above).expect("Valid offset");
                !self.envir.has_opaque_floor(above)
                    && (Level::ZERO <= above.level || self.envir.is_accessible(above))
            })
    }

    fn remember_visible(&self, relative_to: PosOffset, visible: Visible) -> Visible {
        self.visible_cache
            .borrow_mut()
            .insert(relative_to, visible.clone());
        visible
    }

    pub(crate) fn nearby(&self, subzone_level: SubzoneLevel) -> bool {
        let Some(nearby_subzone_level_cache) = &self.nearby_subzone_level_cache else {
            // When updating all positions
            return true;
        };

        *nearby_subzone_level_cache
            .borrow_mut()
            .entry(subzone_level)
            .or_insert_with(|| {
                subzone_level
                    .corners()
                    .iter()
                    .any(|corner| self.nearby_pos(*corner, 1))
            })
    }
}
