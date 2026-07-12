use gameplay_common::ObjectName;
use gameplay_location::Pos;
use gameplay_log::{ProtoLogMessage, Severity};
use gameplay_player::PlayerActionState;
use hud::text_color_expect_full;
use text::{Fragment, Phrase, Subject};
use units::Duration;

#[derive(Debug)]
pub(super) struct AttackNothing {
    pub(super) subject: Subject,
}

impl ProtoLogMessage for AttackNothing {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn phrase(self) -> Phrase {
        self.subject.verb("attack", "s").hard("nothing")
    }
}

#[derive(Debug)]
pub(super) struct CantClose {
    pub(super) subject: Subject,
    pub(super) uncloseable: Fragment,
}

impl ProtoLogMessage for CantClose {
    const SEVERITY: Severity = Severity::Error;

    fn phrase(self) -> Phrase {
        self.subject.simple("can't close").push(self.uncloseable)
    }
}

#[derive(Debug)]
pub(super) struct CantCloseOn {
    pub(super) subject: Subject,
    pub(super) closeable: Fragment,
    pub(super) obstacle: Fragment,
}

impl ProtoLogMessage for CantCloseOn {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn phrase(self) -> Phrase {
        self.subject
            .simple("can't close")
            .push(self.closeable)
            .soft("on")
            .push(self.obstacle)
    }
}

#[derive(Debug)]
pub(super) struct CrashInto<'a> {
    pub(super) subject: Subject,
    pub(super) obstacle: &'a ObjectName,
    pub(super) to: Pos,
}

impl ProtoLogMessage for CrashInto<'_> {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn phrase(self) -> Phrase {
        self.subject
            .verb("crash", "es")
            .soft("into")
            .push(self.obstacle.single(self.to))
    }
}

#[derive(Debug)]
pub(super) struct Drop {
    pub(super) subject: Subject,
    pub(super) item: Vec<Fragment>,
}

impl ProtoLogMessage for Drop {
    const SEVERITY: Severity = Severity::Neutral;

    fn phrase(self) -> Phrase {
        self.subject.verb("drop", "s").extend(self.item)
    }
}

#[derive(Debug)]
pub(super) struct FirstExamineYourDestination;

impl ProtoLogMessage for FirstExamineYourDestination {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn phrase(self) -> Phrase {
        Phrase::new("First examine your destination")
    }
}

#[derive(Debug)]
pub(super) struct HaltAtTheLedge {
    pub(super) subject: Subject,
}

impl ProtoLogMessage for HaltAtTheLedge {
    const SEVERITY: Severity = Severity::Danger;

    fn phrase(self) -> Phrase {
        self.subject.verb("halt", "s").soft("at").hard("the ledge")
    }
}

#[derive(Debug)]
pub(super) struct IsTooExhaustedTo {
    pub(super) subject: Subject,
    pub(super) verb: &'static str,
}

impl ProtoLogMessage for IsTooExhaustedTo {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn phrase(self) -> Phrase {
        self.subject
            .is()
            .hard("too exhausted")
            .soft("to")
            .hard(self.verb)
    }
}

#[derive(Debug)]
pub(super) struct Move {
    pub(super) subject: Subject,
    pub(super) item: Vec<Fragment>,
}

impl ProtoLogMessage for Move {
    const SEVERITY: Severity = Severity::Neutral;

    fn phrase(self) -> Phrase {
        self.subject.verb("move", "s").extend(self.item)
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
pub(super) struct PickUp {
    pub(super) subject: Subject,
    pub(super) taken: Vec<Fragment>,
}

impl ProtoLogMessage for PickUp {
    const SEVERITY: Severity = Severity::Neutral;

    fn phrase(self) -> Phrase {
        self.subject.verb("pick", "s").hard("up").extend(self.taken)
    }
}

#[derive(Debug)]
pub(super) struct PulpNothing {
    pub(super) subject: Subject,
}

impl ProtoLogMessage for PulpNothing {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn phrase(self) -> Phrase {
        self.subject.verb("pulp", "s").hard("nothing")
    }
}

#[derive()]
pub(super) struct SmashInvalid {
    pub(super) subject: Subject,
    pub(super) object: &'static str,
}

impl ProtoLogMessage for SmashInvalid {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn phrase(self) -> Phrase {
        self.subject.verb("smash", "es").hard(self.object)
    }
}

#[derive()]
pub(super) struct SubzoneNotFoundWhileMovingAnItem;

impl ProtoLogMessage for SubzoneNotFoundWhileMovingAnItem {
    const SEVERITY: Severity = Severity::Error;

    fn phrase(self) -> Phrase {
        Phrase::new("Subzone not found when moving an item")
    }
}

#[derive()]
pub(super) struct TooFarToMove;

impl ProtoLogMessage for TooFarToMove {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn phrase(self) -> Phrase {
        Phrase::new("Too far to move")
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
pub(super) struct YouSleepFor {
    pub(super) total_duration: Duration,
}

impl ProtoLogMessage for YouSleepFor {
    const SEVERITY: Severity = Severity::Neutral;

    fn phrase(self) -> Phrase {
        let color = text_color_expect_full(self.total_duration / (Duration::HOUR * 8));

        Self::you("sleep for").push(Fragment::colorized(
            self.total_duration.short_format(),
            color,
        ))
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
