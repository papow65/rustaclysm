use crate::gameplay::{Level, Overzone, Pos, SubzoneLevel, ZoneLevel};
use bevy::platform::collections::HashMap;
use cdda_json_files::{CddaAmount, RepetitionBlock};
use std::hash::Hash;

pub(crate) trait RepetitionBlockExt<'a, T> {
    fn load_as_subzone(&'a self, subzone_level: SubzoneLevel) -> HashMap<Pos, &'a T>;

    #[expect(unused)]
    fn load_as_zone_level(&'a self, zone_level: ZoneLevel) -> HashMap<Pos, &'a T>;

    fn load_as_overzone(&'a self, overzone: Overzone, level: Level) -> HashMap<ZoneLevel, &'a T>;

    fn load<F, X: Eq + Hash>(&'a self, location: F, size: i32) -> HashMap<X, &'a T>
    where
        F: Fn(i32, i32) -> X;
}

impl<T> RepetitionBlockExt<'_, T> for RepetitionBlock<T> {
    fn load_as_subzone(&self, subzone_level: SubzoneLevel) -> HashMap<Pos, &T> {
        self.load(
            |x, z| subzone_level.base_corner().horizontal_offset(x, z),
            12,
        )
    }

    fn load_as_zone_level(&self, zone_level: ZoneLevel) -> HashMap<Pos, &T> {
        let base_pos = zone_level.base_corner();
        self.load(|x, z| base_pos.horizontal_offset(x, z), 24)
    }

    fn load_as_overzone(&self, overzone: Overzone, level: Level) -> HashMap<ZoneLevel, &T> {
        self.load(
            |x, z| overzone.base_zone().offset(x, z).zone_level(level),
            180,
        )
    }

    fn load<F, X: Eq + Hash>(&self, location: F, size: i32) -> HashMap<X, &T>
    where
        F: Fn(i32, i32) -> X,
    {
        let mut result = HashMap::default();
        let mut i: i32 = 0;
        for repetition in &self.0 {
            let CddaAmount { obj, amount } = repetition.as_amount();
            let amount = *amount as i32;
            for j in i..i + amount {
                result.insert(location(j.rem_euclid(size), j.div_euclid(size)), obj);
            }
            i += amount;
        }
        assert_eq!(
            i,
            size * size,
            "The repetion count should be the square of given size"
        );
        assert_eq!(
            result.len(),
            i as usize,
            "The amount of entries in the result should match the repitition count"
        );
        result
    }
}
