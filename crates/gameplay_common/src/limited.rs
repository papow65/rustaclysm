use bevy::prelude::TextColor;
use hud::text_color_expect_full;

#[derive(Clone, Debug)]
pub struct Evolution {
    pub before: u16,
    pub after: u16,
}

impl Evolution {
    #[must_use]
    pub const fn change_abs(&self) -> u16 {
        self.after.abs_diff(self.before)
    }

    #[must_use]
    pub const fn changed(&self) -> bool {
        self.after != self.before
    }
}

#[derive(Debug, PartialEq)]
pub struct Limited {
    pub current: u16,
    pub max: u16,
}

impl Limited {
    #[expect(unused)]
    pub(crate) const fn empty(max: u16) -> Self {
        Self { current: 0, max }
    }

    #[must_use]
    pub const fn full(max: u16) -> Self {
        Self { current: max, max }
    }

    #[must_use]
    pub fn relative(&self) -> f32 {
        f32::from(self.current) / f32::from(self.max)
    }

    pub fn adjust(&mut self, added_amount: i16) -> Evolution {
        let before = self.current;
        self.current = (self.current as i16)
            .saturating_add(added_amount)
            .clamp(0, self.max as i16) as u16;
        Evolution {
            before,
            after: self.current,
        }
    }

    pub fn raise(&mut self, amount: u16) -> Evolution {
        self.adjust(amount as i16)
    }

    pub fn lower(&mut self, amount: u16) -> Evolution {
        self.adjust(-(amount as i16))
    }

    #[must_use]
    pub fn can_add(&self, amount: i16) -> bool {
        matches!((self.current as i16).overflowing_add(amount), (sum, true) if (0_i16..=(self.max as i16)).contains(&sum))
    }

    #[expect(unused)]
    pub(crate) fn can_subtract(&self, amount: i16) -> bool {
        self.can_add(-amount)
    }

    pub(crate) fn try_add(&mut self, amount: i16) -> Result<(), ()> {
        match (self.current as i16).overflowing_add(amount) {
            (sum, true) if (0_i16..=(self.max as i16)).contains(&sum) => {
                self.current = sum as u16;
                Ok(())
            }
            _ => Err(()),
        }
    }

    #[expect(unused)]
    pub(crate) fn try_subtract(&mut self, amount: i16) -> Result<(), ()> {
        self.try_add(-amount)
    }

    #[must_use]
    pub const fn current(&self) -> u16 {
        self.current
    }

    #[expect(unused)]
    pub(crate) const fn max(&self) -> u16 {
        self.max
    }

    #[expect(unused)]
    pub(crate) const fn left(&self) -> u16 {
        self.max - self.current
    }

    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.current == 0
    }

    #[must_use]
    pub const fn is_nonzero(&self) -> bool {
        0 < self.current
    }

    #[must_use]
    pub const fn is_max(&self) -> bool {
        self.current == self.max
    }

    #[must_use]
    pub fn color(&self) -> TextColor {
        text_color_expect_full(self.relative())
    }
}
