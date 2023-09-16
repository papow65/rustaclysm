use crate::prelude::*;
use bevy::prelude::*;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn remove_gameplay_resources(mut commands: Commands) {
    commands.remove_resource::<Infos>();
    commands.remove_resource::<Location>();
    commands.remove_resource::<SubzoneLevelEntities>();
    commands.remove_resource::<ZoneLevelEntities>();
    commands.remove_resource::<InstructionQueue>();
    commands.remove_resource::<AppearanceCache>();
    commands.remove_resource::<MeshCaches>();
    commands.remove_resource::<VisualizationUpdate>();
    commands.remove_resource::<Explored>();
    commands.remove_resource::<Sav>();
    commands.remove_resource::<Timeouts>();
    commands.remove_resource::<ZoneLevelIds>();
    commands.remove_resource::<CameraOffset>();
    commands.remove_resource::<InstructionQueue>();
    commands.remove_resource::<PlayerActionState>();
    commands.remove_resource::<StatusTextSections>();
    commands.remove_resource::<Events<Message>>();
    commands.remove_resource::<Events<ActorEvent<Stay>>>();
    commands.remove_resource::<Events<ActorEvent<Step>>>();
    commands.remove_resource::<Events<ActorEvent<Attack>>>();
    commands.remove_resource::<Events<ActorEvent<Smash>>>();
    commands.remove_resource::<Events<ActorEvent<Close>>>();
    commands.remove_resource::<Events<ActorEvent<Wield>>>();
    commands.remove_resource::<Events<ActorEvent<Unwield>>>();
    commands.remove_resource::<Events<ActorEvent<Pickup>>>();
    commands.remove_resource::<Events<ActorEvent<Dump>>>();
    commands.remove_resource::<Events<ActorEvent<ExamineItem>>>();
    commands.remove_resource::<Events<ActorEvent<ChangePace>>>();
    commands.remove_resource::<Events<ActorEvent<StaminaImpact>>>();
    commands.remove_resource::<Events<ActorEvent<Timeout>>>();
    commands.remove_resource::<Events<ActorEvent<Damage>>>();
    commands.remove_resource::<Events<ActorEvent<Healing>>>();
    commands.remove_resource::<Events<ItemEvent<Damage>>>();
    commands.remove_resource::<Events<TerrainEvent<Toggle>>>();
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn disable_screen_state(mut next_state: ResMut<NextState<GameplayScreenState>>) {
    next_state.set(GameplayScreenState::Inapplicable);
}
