use text::Subject;

#[derive(Clone, Debug)]
pub struct Damage {
    // TODO damage types
    pub attacker: Subject, // for logging
    pub amount: u16,
}
