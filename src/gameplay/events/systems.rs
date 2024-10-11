use crate::gameplay::events::{
    ActorEvent, CorpseEvent, Damage, DespawnSubzoneLevel, DespawnZoneLevel, Healing, Message,
    RefreshAfterBehavior, SpawnSubzoneLevel, SpawnZoneLevel, TerrainEvent,
    UpdateZoneLevelVisibility,
};
use crate::gameplay::Toggle;
use bevy::prelude::{Commands, Event, Events, IntoSystemConfigs, ResMut};

pub(super) fn create_event_resources(mut commands: Commands) {
    commands.insert_resource(Events::<ActorEvent<Damage>>::default());
    commands.insert_resource(Events::<ActorEvent<Healing>>::default());
    commands.insert_resource(Events::<CorpseEvent<Damage>>::default());
    commands.insert_resource(Events::<DespawnSubzoneLevel>::default());
    commands.insert_resource(Events::<DespawnZoneLevel>::default());
    commands.insert_resource(Events::<Message>::default());
    commands.insert_resource(Events::<RefreshAfterBehavior>::default());
    commands.insert_resource(Events::<SpawnSubzoneLevel>::default());
    commands.insert_resource(Events::<SpawnZoneLevel>::default());
    commands.insert_resource(Events::<TerrainEvent<Damage>>::default());
    commands.insert_resource(Events::<TerrainEvent<Toggle>>::default());
    commands.insert_resource(Events::<UpdateZoneLevelVisibility>::default());
}

/// Bevy 0.14.1 (and earlier versions) can't properly deal with event resources being deleted and recreated later.
/// So we clear the event resources instead.
pub(super) fn clear_event_resources() -> impl IntoSystemConfigs<()> {
    (
        clear_events::<ActorEvent<Damage>>,
        clear_events::<ActorEvent<Healing>>,
        clear_events::<CorpseEvent<Damage>>,
        clear_events::<DespawnSubzoneLevel>,
        clear_events::<DespawnZoneLevel>,
        clear_events::<Message>,
        clear_events::<RefreshAfterBehavior>,
        clear_events::<SpawnSubzoneLevel>,
        clear_events::<SpawnZoneLevel>,
        clear_events::<TerrainEvent<Damage>>,
        clear_events::<TerrainEvent<Toggle>>,
        clear_events::<UpdateZoneLevelVisibility>,
    )
        .chain() // for a simple return type
}

fn clear_events<T: Event>(mut events: ResMut<Events<T>>) {
    events.clear();
}
