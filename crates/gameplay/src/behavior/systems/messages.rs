use crate::{Evolution, ProtoLogMessage, Severity};
use text::{Fragment, Phrase, Subject};

#[derive(Debug)]
pub(super) struct Break {
    pub(super) breaker: Subject,
    pub(super) broken: Fragment,
}

impl ProtoLogMessage for Break {
    const SEVERITY: Severity = Severity::Danger;

    fn phrase(self) -> Phrase {
        self.breaker.verb("break", "s").push(self.broken)
    }
}

#[derive(Debug)]
pub(super) struct Heal {
    pub(super) subject: Subject,
    pub(super) evolution: Evolution,
}

impl ProtoLogMessage for Heal {
    const SEVERITY: Severity = Severity::Success;

    fn phrase(self) -> Phrase {
        let builder = self.subject.verb("heal", "s");
        if self.evolution.change_abs() == 1 {
            builder.push(Fragment::good("a bit"))
        } else {
            builder
                .soft("for")
                .push(Fragment::good(format!("{}", self.evolution.change_abs())))
                .soft(format!(
                    "({} -> {})",
                    self.evolution.before, self.evolution.after
                ))
        }
    }
}

#[derive(Debug)]
pub(super) struct Hit {
    pub(super) attacker: Subject,
    pub(super) object: Fragment, // sentence object: a character, terrain, or furniture
    pub(super) evolution: Evolution,
}

impl ProtoLogMessage for Hit {
    const SEVERITY: Severity = Severity::Danger;

    fn phrase(self) -> Phrase {
        let builder = self.attacker.verb("hit", "s").push(self.object);
        if self.evolution.changed() {
            builder
                .soft("for")
                .push(Fragment::warn(format!("{}", self.evolution.change_abs())))
                .soft(format!(
                    "({} -> {})",
                    self.evolution.before, self.evolution.after
                ))
        } else {
            builder.soft("but it has").hard("no effect")
        }
    }
}

#[derive(Debug)]
pub(super) struct IsThoroughlyPulped {
    pub(super) corpse: Fragment,
}

impl ProtoLogMessage for IsThoroughlyPulped {
    const SEVERITY: Severity = Severity::Success;

    fn phrase(self) -> Phrase {
        Subject::Other(Phrase::from_fragment(self.corpse))
            .is()
            .hard("thoroughly pulped")
    }
}

#[derive(Debug)]
pub(super) struct Kill {
    pub(super) killer: Subject,
    pub(super) killed: Fragment,
}

impl ProtoLogMessage for Kill {
    const SEVERITY: Severity = Severity::Danger;

    fn phrase(self) -> Phrase {
        self.killer.verb("kill", "s").push(self.killed)
    }
}

#[derive(Debug)]
pub(super) struct NpcActionFailed;

impl ProtoLogMessage for NpcActionFailed {
    const SEVERITY: Severity = Severity::Error;

    fn phrase(self) -> Phrase {
        Phrase::new("NPC action failed")
    }
}

#[derive(Debug)]
pub(super) struct Pulp {
    pub(super) pulper: Subject,
    pub(super) corpse: Fragment,
}

impl ProtoLogMessage for Pulp {
    const SEVERITY: Severity = Severity::Neutral;

    fn phrase(self) -> Phrase {
        self.pulper.verb("pulp", "s").push(self.corpse)
    }
}
