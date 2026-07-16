use gameplay_common::Visible;
use gameplay_model::LastSeen;
use gameplay_object::Mobile;
use gameplay_player::Player;

pub(crate) trait LastSeenExt {
    fn update(&mut self, visible: Visible);
    fn shown(&self, player: Option<&Player>, mobile: Option<&Mobile>) -> bool;
}

impl LastSeenExt for LastSeen {
    fn update(&mut self, visible: Visible) {
        if visible == Visible::Seen {
            *self = Self::Currently;
        } else if self == &Self::Currently {
            *self = Self::Previously;
        }
    }

    fn shown(&self, player: Option<&Player>, mobile: Option<&Mobile>) -> bool {
        // Things that can move, like NPCs, are hidden when out of sight.
        self == &Self::Currently
            || (self == &Self::Previously && mobile.is_none())
            || player.is_some()
    }
}
