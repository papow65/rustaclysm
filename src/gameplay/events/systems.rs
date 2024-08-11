use crate::gameplay::events::{
    CorpseEvent, Damage, DespawnSubzoneLevel, DespawnZoneLevel, Message, RefreshAfterBehavior,
    SpawnSubzoneLevel, SpawnZoneLevel, TerrainEvent, UpdateZoneLevelVisibility,
};
use crate::gameplay::Toggle;
use bevy::prelude::{Commands, Events};

pub(super) fn create_event_resources(mut commands: Commands) {
    commands.insert_resource(Events::<Message>::default());
    commands.insert_resource(Events::<SpawnSubzoneLevel>::default());
    commands.insert_resource(Events::<DespawnSubzoneLevel>::default());
    commands.insert_resource(Events::<SpawnZoneLevel>::default());
    commands.insert_resource(Events::<UpdateZoneLevelVisibility>::default());
    commands.insert_resource(Events::<DespawnZoneLevel>::default());
    commands.insert_resource(Events::<CorpseEvent<Damage>>::default());
    commands.insert_resource(Events::<TerrainEvent<Damage>>::default());
    commands.insert_resource(Events::<TerrainEvent<Toggle>>::default());
    commands.insert_resource(Events::<RefreshAfterBehavior>::default());
}

pub(super) fn remove_event_resources(mut commands: Commands) {
    commands.remove_resource::<Events<Message>>();
    commands.remove_resource::<Events<SpawnSubzoneLevel>>();
    commands.remove_resource::<Events<DespawnSubzoneLevel>>();
    commands.remove_resource::<Events<SpawnZoneLevel>>();
    commands.remove_resource::<Events<UpdateZoneLevelVisibility>>();
    commands.remove_resource::<Events<DespawnZoneLevel>>();
    commands.remove_resource::<Events<CorpseEvent<Damage>>>();
    commands.remove_resource::<Events<TerrainEvent<Damage>>>();
    commands.remove_resource::<Events<TerrainEvent<Toggle>>>();
    commands.remove_resource::<Events<RefreshAfterBehavior>>();
}
