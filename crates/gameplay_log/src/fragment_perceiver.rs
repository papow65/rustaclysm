use gameplay_location::Pos;

pub trait PosPerceiver {
    /// Check perception of a position by the player
    fn can_perceive(&self, pos: Pos) -> bool;
}
