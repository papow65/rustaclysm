use crate::prelude::text_color;
use bevy::prelude::Color;

#[derive(Clone, Debug)]
pub(crate) struct Evolution {
    pub(crate) before: u16,
    pub(crate) after: u16,
}

impl Evolution {
    pub(crate) const fn change_abs(&self) -> u16 {
        self.after.abs_diff(self.before)
    }

    pub(crate) const fn changed(&self) -> bool {
        self.after != self.before
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Limited {
    current: u16,
    max: u16,
}

impl Limited {
    #[allow(unused)]
    pub(crate) const fn empty(max: u16) -> Self {
        Self { current: 0, max }
    }

    pub(crate) const fn full(max: u16) -> Self {
        Self { current: max, max }
    }

    pub(crate) fn relative(&self) -> f32 {
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

    pub(crate) const fn current(&self) -> u16 {
        self.current
    }

    #[allow(unused)]
    pub(crate) const fn max(&self) -> u16 {
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
        text_color(self.relative())
    }
}
