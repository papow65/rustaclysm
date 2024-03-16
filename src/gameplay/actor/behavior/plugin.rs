use super::{
    schedule::BehaviorSchedule,
    systems::{
        core::{egible_character, perform_action, plan_action, proces_impact},
        handlers::{
            combine_items, toggle_doors, update_corpses, update_damaged_characters,
            update_damaged_corpses, update_damaged_terrain, update_explored,
            update_healed_characters, update_stamina,
        },
    },
};
use crate::prelude::{
    ActorEvent, CorpseEvent, Damage, Healing, StaminaImpact, TerrainEvent, Toggle,
};
use bevy::prelude::{on_event, App, IntoSystem, IntoSystemConfigs, Plugin};

pub(crate) struct BehaviorPlugin;

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(BehaviorSchedule);

        app.add_systems(
            BehaviorSchedule,
            (
                egible_character
                    .pipe(plan_action)
                    .pipe(perform_action)
                    .pipe(proces_impact),
                (
                    (
                        // actor events
                        // Make sure killed actors are handled early
                        update_damaged_characters.run_if(on_event::<ActorEvent<Damage>>()),
                        (
                            update_stamina.run_if(on_event::<ActorEvent<StaminaImpact>>()),
                            update_healed_characters.run_if(on_event::<ActorEvent<Healing>>()),
                            update_corpses,
                            update_explored,
                        ),
                    )
                        .chain(),
                    (
                        // item events
                        update_damaged_corpses.run_if(on_event::<CorpseEvent<Damage>>()),
                        combine_items,
                    )
                        .chain(),
                    (
                        // terrain events
                        // Make sure destoyed items are handled early
                        update_damaged_terrain.run_if(on_event::<TerrainEvent<Damage>>()),
                        toggle_doors.run_if(on_event::<TerrainEvent<Toggle>>()),
                    )
                        .chain(),
                ),
            )
                .chain(),
        );
    }
}
