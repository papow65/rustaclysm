use super::systems::{
    core::run_behavior_schedule,
    refresh::{
        check_items, update_hidden_item_visibility, update_transforms,
        update_visualization_on_player_move, update_visualization_on_weather_change,
    },
};
use crate::prelude::{update_visualization_on_item_move, RefreshAfterBehavior, Timeouts};
use bevy::prelude::{on_event, resource_exists_and_changed, IntoSystemConfigs};

pub(crate) fn behavior_systems() -> impl IntoSystemConfigs<()> {
    (
        run_behavior_schedule,
        (
            update_transforms,
            update_hidden_item_visibility,
            update_visualization_on_item_move,
            update_visualization_on_player_move,
            update_visualization_on_weather_change.run_if(resource_exists_and_changed::<Timeouts>),
            #[cfg(debug_assertions)]
            check_items,
        )
            .run_if(on_event::<RefreshAfterBehavior>()),
    )
        .chain()
}
