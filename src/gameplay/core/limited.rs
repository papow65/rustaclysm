use crate::prelude::{Partial, BAD_TEXT_COLOR, GOOD_TEXT_COLOR, WARN_TEXT_COLOR};
use bevy::prelude::Color;

#[derive(Clone, Debug)]
pub(crate) struct Evolution {
    pub(crate) before: u16,
    pub(crate) after: u16,
}

impl Evolution {
    pub(crate) fn changed(&self) -> bool {
        self.before != self.after
    }
}

#[derive(Debug)]
pub(crate) struct Limited {
    current: u16,
    max: u16,
}

impl Limited {
    #[allow(unused)]
    pub(crate) fn empty(max: u16) -> Self {
        Self { current: 0, max }
    }

    pub(crate) fn full(max: u16) -> Self {
        Self { current: max, max }
    }

    #[allow(unused)]
    pub(crate) fn new_in_fraction_order(current: u16, max: u16) -> Self {
        assert!(current <= max);
        Self { current, max }
    }

    pub(crate) const fn partial(&self) -> Partial {
        Partial::from_u8((255_u32 * self.current as u32 / self.max as u32) as u8)
    }

    pub(crate) fn percent(&self) -> f32 {
        f32::from(self.current) / f32::from(self.max)
    }

    pub(crate) fn adjust(&mut self, added_amount: i16) -> Evolution {
        let before = self.current;
        self.current = (self.current as i16)
            .saturating_add(added_amount)
            .clamp(0, self.max as i16) as u16;
        Evolution {
            before,
            after: self.current,
        }
    }

    pub(crate) fn raise(&mut self, amount: u16) -> Evolution {
        self.adjust(amount as i16)
    }

    pub(crate) fn lower(&mut self, amount: u16) -> Evolution {
        self.adjust(-(amount as i16))
    }

    #[allow(unused)]
    pub(crate) fn can_add(&mut self, amount: i16) -> bool {
        matches!((self.current as i16).overflowing_add(amount), (sum, true) if (0_i16..=(self.max as i16)).contains(&sum))
    }

    #[allow(unused)]
    pub(crate) fn can_subtract(&mut self, amount: i16) -> bool {
        self.can_add(-amount)
    }

    #[allow(unused)]
    pub(crate) fn try_add(&mut self, amount: i16) -> Result<(), ()> {
        match (self.current as i16).overflowing_add(amount) {
            (sum, true) if (0_i16..=(self.max as i16)).contains(&sum) => {
                self.current = sum as u16;
                Ok(())
            }
            _ => Err(()),
        }
    }

    #[allow(unused)]
    pub(crate) fn try_subtract(&mut self, amount: i16) -> Result<(), ()> {
        self.try_add(-amount)
    }

    pub(crate) fn current(&self) -> u16 {
        self.current
    }

    #[allow(unused)]
    pub(crate) fn max(&self) -> u16 {
        self.max
    }

    #[allow(unused)]
    pub(crate) const fn left(&self) -> u16 {
        self.max - self.current
    }

    pub(crate) const fn is_zero(&self) -> bool {
        self.current == 0
    }

    #[allow(unused)]
    pub(crate) const fn is_nonzero(&self) -> bool {
        0 < self.current
    }

    #[allow(unused)]
    pub(crate) const fn is_max(&self) -> bool {
        self.current == self.max
    }

    pub(crate) fn color(&self) -> Color {
        let percent = self.percent();
        let (part, min_color, max_color) = if 0.5 <= percent {
            (2.0 * percent - 1.0, WARN_TEXT_COLOR, GOOD_TEXT_COLOR)
        } else {
            (2.0 * percent, BAD_TEXT_COLOR, WARN_TEXT_COLOR)
        };

        Color::rgb(
            min_color.r() + part * (max_color.r() - min_color.r()),
            min_color.g() + part * (max_color.g() - min_color.g()),
            min_color.b() + part * (max_color.b() - min_color.b()),
        )
    }
}
