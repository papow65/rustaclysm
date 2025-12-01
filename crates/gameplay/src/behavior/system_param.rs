use crate::{
    Actor, Envir, GameplayScreenState, Player, PlayerActionState, PlayerInstructions, Timeouts,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Entity, Query, Res, Single, State, With};

#[derive(SystemParam)]
pub(crate) struct BehaviorValidator<'w, 's> {
    player_action_state: Option<Res<'w, State<PlayerActionState>>>,
    gameplay_screen_state: Option<Res<'w, State<GameplayScreenState>>>,
    envir: Option<Envir<'w, 's>>,
    timeouts: Option<Res<'w, Timeouts>>,
    player_instructions: Option<Res<'w, PlayerInstructions>>,
    actors: Query<'w, 's, Actor>,
    player: Option<Single<'w, 's, Entity, With<Player>>>,
}

impl BehaviorValidator<'_, '_> {
    /// The next actor can behave
    pub(super) fn looping_behavior(&self) -> bool {
        let (
            Some(player_action_state),
            Some(gameplay_screen_state),
            Some(envir),
            Some(timeouts),
            Some(player_instructions),
            Some(player),
        ) = (
            self.player_action_state.as_ref(),
            self.gameplay_screen_state.as_ref(),
            self.envir.as_ref(),
            self.timeouts.as_ref(),
            self.player_instructions.as_ref(),
            self.player.as_ref(),
        )
        else {
            return false;
        };

        if !gameplay_screen_state.allow_behavior() {
            return false;
        }

        player_action_state.is_automatic() || !player_instructions.is_empty() || {
            let egible_entities = self
                .actors
                .iter()
                .filter(|a| envir.is_accessible(*a.pos))
                .map(|a| a.entity)
                .collect::<Vec<_>>();
            !timeouts.is_player_next(**player, &egible_entities)
        }
    }
}
