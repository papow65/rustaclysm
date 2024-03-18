use super::{
    schedule::BehaviorSchedule,
    systems::refresh::{
        update_hidden_item_visibility, update_transforms, update_visualization_on_player_move,
        update_visualization_on_weather_change,
    },
};
use crate::prelude::{update_visualization_on_item_move, RefreshAfterBehavior};
use bevy::prelude::{on_event, IntoSystemConfigs};

#[cfg(debug_assertions)]
use super::systems::refresh::check_items;

pub(crate) fn behavior_systems() -> impl IntoSystemConfigs<()> {
    (
        BehaviorSchedule::repeat,
        (
            update_transforms,
            update_hidden_item_visibility,
            update_visualization_on_item_move,
            update_visualization_on_player_move,
            update_visualization_on_weather_change,
            #[cfg(debug_assertions)]
            check_items,
        )
            .run_if(on_event::<RefreshAfterBehavior>()),
    )
        .chain()
}
