use gameplay_log::{ProtoLogMessage, Severity};
use gameplay_player::PlayerActionState;
use hud::text_color_expect_full;
use text::{Fragment, Phrase, Subject};
use units::Duration;

#[derive(Debug)]
pub(super) struct FirstExamineYourDestination;

impl ProtoLogMessage for FirstExamineYourDestination {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn phrase(self) -> Phrase {
        Phrase::new("First examine your destination")
    }
}

#[derive(Debug)]
pub(super) struct NoPlaceToCraftNearby;

impl ProtoLogMessage for NoPlaceToCraftNearby {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn phrase(self) -> Phrase {
        Phrase::new("no place to craft nearby")
    }
}

#[derive(Debug)]
pub(super) struct NoTargetsNearby;

impl ProtoLogMessage for NoTargetsNearby {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn phrase(self) -> Phrase {
        Phrase::new("no targets nearby")
    }
}

#[derive(Debug)]
pub(super) struct NothingToCloseNearby;

impl ProtoLogMessage for NothingToCloseNearby {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn phrase(self) -> Phrase {
        Phrase::new("nothing to close nearby")
    }
}

#[derive(Debug)]
pub(super) struct YouAreAlmostOutOfBreathAndStop {
    pub(crate) verb: String,
}

impl ProtoLogMessage for YouAreAlmostOutOfBreathAndStop {
    const SEVERITY: Severity = Severity::Danger;

    fn phrase(self) -> Phrase {
        Subject::You
            .is()
            .hard("almost out of breath")
            .soft("and")
            .hard("stop")
            .hard(self.verb)
    }
}

#[derive(Debug)]
pub(super) struct YouAreStillAsleep;

impl ProtoLogMessage for YouAreStillAsleep {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn phrase(self) -> Phrase {
        Self::you("are still asleep. Zzz...")
    }
}

#[derive(Debug)]
pub(super) struct YouAreStillDraggingItems;

impl ProtoLogMessage for YouAreStillDraggingItems {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn phrase(self) -> Phrase {
        Self::you("are still dragging items")
    }
}

#[derive(Debug)]
pub(super) struct YouCantAttackYourself;

impl ProtoLogMessage for YouCantAttackYourself {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn phrase(self) -> Phrase {
        Self::you("can't attack yourself")
    }
}

#[derive(Debug)]
pub(super) struct YouCant {
    pub(super) verb: &'static str,
    pub(super) direction: &'static str,
}

impl ProtoLogMessage for YouCant {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn phrase(self) -> Phrase {
        Self::you("can't").hard(self.verb).hard(self.direction)
    }
}

#[derive(Debug)]
pub(super) struct YouFallAsleep;

impl ProtoLogMessage for YouFallAsleep {
    const SEVERITY: Severity = Severity::Neutral;

    fn phrase(self) -> Phrase {
        Self::you("fall asleep... Zzz...")
    }
}

#[derive(Debug)]
pub(super) struct YouFinish<const SUCCESS: bool> {
    pub(super) action: PlayerActionState,
}

impl<const SUCCESS: bool> YouFinish<SUCCESS> {
    const fn severity() -> Severity {
        if SUCCESS {
            Severity::Success
        } else {
            Severity::Neutral
        }
    }
}

impl<const SUCCESS: bool> ProtoLogMessage for YouFinish<SUCCESS> {
    const SEVERITY: Severity = Self::severity();

    fn phrase(self) -> Phrase {
        Self::you("finish").hard(if let PlayerActionState::Crafting { .. } = self.action {
            String::from("your craft")
        } else {
            self.action.to_string().to_lowercase()
        })
    }
}

#[derive(Debug)]
pub(super) struct YouSpotAndStop<S: Into<String>> {
    pub(super) seen: Fragment,
    pub(super) verb: S,
}

impl<S: Into<String>> ProtoLogMessage for YouSpotAndStop<S> {
    const SEVERITY: Severity = Severity::Danger;

    fn phrase(self) -> Phrase {
        Self::you("spot")
            .push(self.seen)
            .soft("and")
            .hard("stop")
            .hard(self.verb)
    }
}

#[derive(Debug)]
pub(super) struct YouStartDefending;

impl ProtoLogMessage for YouStartDefending {
    const SEVERITY: Severity = Severity::Danger;

    fn phrase(self) -> Phrase {
        Self::you("start defending...")
    }
}

#[derive(Debug)]
pub(super) struct YouWakeUpAfterSleeping {
    pub(super) duration: Duration,
}

impl ProtoLogMessage for YouWakeUpAfterSleeping {
    const SEVERITY: Severity = Severity::Neutral;

    fn phrase(self) -> Phrase {
        let color = text_color_expect_full(self.duration / (Duration::HOUR * 8));

        Self::you("wake up after sleeping")
            .push(Fragment::colorized(self.duration.short_format(), color))
    }
}
