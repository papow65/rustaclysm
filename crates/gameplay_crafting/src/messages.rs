use crate::Craft;
use gameplay_log::{ProtoLogMessage, Severity};
use hud::text_color_expect_full;
use text::{Fragment, Phrase};

#[derive(Debug)]
pub struct CraftProgressLeft<'a> {
    pub craft: &'a Craft,
}

impl ProtoLogMessage for CraftProgressLeft<'_> {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn phrase(self) -> Phrase {
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
