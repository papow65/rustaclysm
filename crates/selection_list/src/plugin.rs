use crate::systems::{
    create_selection_list_key_bindings, scroll_to_selection, select_first_when_empty,
};
use crate::{SelectedItemOf, SelectionListItems};
use bevy::ecs::query::QueryFilter;
use bevy::prelude::{App, IntoScheduleConfigs as _, OnEnter, States, Update, in_state};
use util::when_changed;

pub fn selection_list_plugin<S: States + Default, Q: QueryFilter + 'static>(
    app: &mut App,
    state: S,
    select_element_label: &'static str,
) {
    app.add_systems(
        OnEnter(state.clone()),
        create_selection_list_key_bindings::<S, Q>(state.clone(), select_element_label),
    );

    app.add_systems(
        Update,
        (
            select_first_when_empty.run_if(when_changed::<SelectionListItems>),
            scroll_to_selection::<Q>.run_if(when_changed::<SelectedItemOf>),
        )
            .run_if(in_state(state)),
    );
}
