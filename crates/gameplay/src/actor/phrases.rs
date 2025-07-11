use crate::actor::PlayerActionState;
use crate::{Craft, Fragment, ObjectName, Phrase, ProtoPhrase, Severity, Subject};
use gameplay_location::Pos;
use hud::text_color_expect_full;
use units::Duration;

#[derive(Debug)]
pub(super) struct AttackNothing {
    pub(super) subject: Subject,
}

impl ProtoPhrase for AttackNothing {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn compose(self) -> Phrase {
        self.subject.verb("attack", "s").hard("nothing")
    }
}

#[derive(Debug)]
pub(super) struct CantClose {
    pub(super) subject: Subject,
    pub(super) uncloseable: Fragment,
}

impl ProtoPhrase for CantClose {
    const SEVERITY: Severity = Severity::Error;

    fn compose(self) -> Phrase {
        self.subject.simple("can't close").push(self.uncloseable)
    }
}

#[derive(Debug)]
pub(super) struct CantCloseOn {
    pub(super) subject: Subject,
    pub(super) closeable: Fragment,
    pub(super) obstacle: Fragment,
}

impl ProtoPhrase for CantCloseOn {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn compose(self) -> Phrase {
        self.subject
            .simple("can't close")
            .push(self.closeable)
            .soft("on")
            .push(self.obstacle)
    }
}

#[derive(Debug)]
pub(super) struct CraftProgressLeft<'a> {
    pub(super) craft: &'a Craft,
}

impl ProtoPhrase for CraftProgressLeft<'_> {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn compose(self) -> Phrase {
        let percent_progress = self.craft.percent_progress();
        let color = text_color_expect_full(percent_progress / 100.0);
        let percent_progress = format!("{percent_progress:.1}");
        let time_left = self.craft.time_left().short_format();

        Phrase::new("Craft:")
            .push(Fragment::colorized(percent_progress, color))
            .hard("% progress -")
            .push(Fragment::colorized(time_left, color))
            .hard("left")
    }
}

#[derive(Debug)]
pub(super) struct CrashInto<'a> {
    pub(super) subject: Subject,
    pub(super) obstacle: &'a ObjectName,
    pub(super) to: Pos,
}

impl ProtoPhrase for CrashInto<'_> {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn compose(self) -> Phrase {
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

impl ProtoPhrase for Drop {
    const SEVERITY: Severity = Severity::Neutral;

    fn compose(self) -> Phrase {
        self.subject.verb("drop", "s").extend(self.item)
    }
}

#[derive(Debug)]
pub(super) struct FirstExamineYourDestination;

impl ProtoPhrase for FirstExamineYourDestination {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn compose(self) -> Phrase {
        Phrase::new("First examine your destination")
    }
}

#[derive(Debug)]
pub(super) struct HaltAtTheLedge {
    pub(super) subject: Subject,
}

impl ProtoPhrase for HaltAtTheLedge {
    const SEVERITY: Severity = Severity::Danger;

    fn compose(self) -> Phrase {
        self.subject.verb("halt", "s").soft("at").hard("the ledge")
    }
}

#[derive(Debug)]
pub(super) struct IsTooExhaustedTo {
    pub(super) subject: Subject,
    pub(super) verb: &'static str,
}

impl ProtoPhrase for IsTooExhaustedTo {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn compose(self) -> Phrase {
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

impl ProtoPhrase for Move {
    const SEVERITY: Severity = Severity::Neutral;

    fn compose(self) -> Phrase {
        self.subject.verb("move", "s").extend(self.item)
    }
}

#[derive(Debug)]
pub(super) struct NoPlaceToCraftNearby;

impl ProtoPhrase for NoPlaceToCraftNearby {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn compose(self) -> Phrase {
        Phrase::new("no place to craft nearby")
    }
}

#[derive(Debug)]
pub(super) struct NoTargetsNearby;

impl ProtoPhrase for NoTargetsNearby {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn compose(self) -> Phrase {
        Phrase::new("no targets nearby")
    }
}

#[derive(Debug)]
pub(super) struct NothingToCloseNearby;

impl ProtoPhrase for NothingToCloseNearby {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn compose(self) -> Phrase {
        Phrase::new("nothing to close nearby")
    }
}

#[derive(Debug)]
pub(super) struct PickUp {
    pub(super) subject: Subject,
    pub(super) taken: Vec<Fragment>,
}

impl ProtoPhrase for PickUp {
    const SEVERITY: Severity = Severity::Neutral;

    fn compose(self) -> Phrase {
        self.subject.verb("pick", "s").hard("up").extend(self.taken)
    }
}

#[derive(Debug)]
pub(super) struct PulpNothing {
    pub(super) subject: Subject,
}

impl ProtoPhrase for PulpNothing {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn compose(self) -> Phrase {
        self.subject.verb("pulp", "s").hard("nothing")
    }
}

#[derive()]
pub(super) struct SmashInvalid {
    pub(super) subject: Subject,
    pub(super) object: &'static str,
}

impl ProtoPhrase for SmashInvalid {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn compose(self) -> Phrase {
        self.subject.verb("smash", "es").hard(self.object)
    }
}

#[derive()]
pub(super) struct SubzoneNotFoundWhileMovingAnItem;

impl ProtoPhrase for SubzoneNotFoundWhileMovingAnItem {
    const SEVERITY: Severity = Severity::Error;

    fn compose(self) -> Phrase {
        Phrase::new("Subzone not found when moving an item")
    }
}

#[derive()]
pub(super) struct TooFarToMove;

impl ProtoPhrase for TooFarToMove {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn compose(self) -> Phrase {
        Phrase::new("Too far to move")
    }
}

#[derive(Debug)]
pub(super) struct YouAreAlmostOutOfBreathAndStop {
    pub(crate) verb: String,
}

impl ProtoPhrase for YouAreAlmostOutOfBreathAndStop {
    const SEVERITY: Severity = Severity::Danger;

    fn compose(self) -> Phrase {
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

impl ProtoPhrase for YouAreStillAsleep {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn compose(self) -> Phrase {
        Self::you("are still asleep. Zzz...")
    }
}

#[derive(Debug)]
pub(super) struct YouAreStillDraggingItems;

impl ProtoPhrase for YouAreStillDraggingItems {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn compose(self) -> Phrase {
        Self::you("are still dragging items")
    }
}

#[derive(Debug)]
pub(super) struct YouCantAttackYourself;

impl ProtoPhrase for YouCantAttackYourself {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn compose(self) -> Phrase {
        Self::you("can't attack yourself")
    }
}

#[derive(Debug)]
pub(super) struct YouCant {
    pub(super) verb: &'static str,
    pub(super) direction: &'static str,
}

impl ProtoPhrase for YouCant {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn compose(self) -> Phrase {
        Self::you("can't").hard(self.verb).hard(self.direction)
    }
}

#[derive(Debug)]
pub(super) struct YouFallAsleep;

impl ProtoPhrase for YouFallAsleep {
    const SEVERITY: Severity = Severity::Neutral;

    fn compose(self) -> Phrase {
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

impl<const SUCCESS: bool> ProtoPhrase for YouFinish<SUCCESS> {
    const SEVERITY: Severity = Self::severity();

    fn compose(self) -> Phrase {
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

impl ProtoPhrase for YouSleepFor {
    const SEVERITY: Severity = Severity::Neutral;

    fn compose(self) -> Phrase {
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

impl<S: Into<String>> ProtoPhrase for YouSpotAndStop<S> {
    const SEVERITY: Severity = Severity::Danger;

    fn compose(self) -> Phrase {
        Self::you("spot")
            .push(self.seen)
            .soft("and")
            .hard("stop")
            .hard(self.verb)
    }
}

#[derive(Debug)]
pub(super) struct YouStartDefending;

impl ProtoPhrase for YouStartDefending {
    const SEVERITY: Severity = Severity::Danger;

    fn compose(self) -> Phrase {
        Self::you("start defending...")
    }
}

#[derive(Debug)]
pub(super) struct YouWait;

impl ProtoPhrase for YouWait {
    const SEVERITY: Severity = Severity::Neutral;

    fn compose(self) -> Phrase {
        Self::you("wait...")
    }
}

#[derive(Debug)]
pub(super) struct YouWakeUpAfterSleeping {
    pub(super) duration: Duration,
}

impl ProtoPhrase for YouWakeUpAfterSleeping {
    const SEVERITY: Severity = Severity::Neutral;

    fn compose(self) -> Phrase {
        let color = text_color_expect_full(self.duration / (Duration::HOUR * 8));

        Self::you("wake up after sleeping")
            .push(Fragment::colorized(self.duration.short_format(), color))
    }
}
