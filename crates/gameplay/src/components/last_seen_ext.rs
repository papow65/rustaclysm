use crate::{BaseSpeed, Player, Visible};
use gameplay_model::LastSeen;

pub(crate) trait LastSeenExt {
    fn update(&mut self, visible: Visible);
    fn shown(&self, player: Option<&Player>, speed: Option<&BaseSpeed>) -> bool;
}

impl LastSeenExt for LastSeen {
    fn update(&mut self, visible: Visible) {
        if visible == Visible::Seen {
            *self = Self::Currently;
        } else if self == &Self::Currently {
            *self = Self::Previously;
        }
    }

    fn shown(&self, player: Option<&Player>, speed: Option<&BaseSpeed>) -> bool {
        // Things that can move, like NPCs, are hidden when out of sight.
        self == &Self::Currently
            || (self == &Self::Previously && speed.is_none())
            || player.is_some()
    }
}
