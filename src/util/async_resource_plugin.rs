use bevy::prelude::{
    not, resource_exists, App, Commands, IntoSystemConfigs, Plugin, ResMut, Resource, Update,
};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future::{block_on, poll_once};
use regex::Regex;
use std::{any::type_name, future::Future, marker::PhantomData, sync::LazyLock, time::Instant};

/// Resources that take a while to load, are loaded in the background, independent of the current [`ApplicationState`](`crate::application::ApplicationState`)
pub(crate) trait AsyncNew<T>: Resource {
    fn async_new() -> impl Future<Output = T> + Send;
}

/// Load the [`AsyncNew`] resource in the background
pub(crate) struct AsyncResourcePlugin<T: AsyncNew<T>>(PhantomData<T>);

impl<T: AsyncNew<T>> Default for AsyncResourcePlugin<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: AsyncNew<T>> Plugin for AsyncResourcePlugin<T> {
    fn build(&self, app: &mut App) {
        app.insert_resource(AsyncResourceLoader::<T>::default());

        app.add_systems(
            Update,
            create_async_resource::<T>.run_if(not(resource_exists::<T>)),
        );
    }
}

#[derive(Debug, Resource)]
pub(crate) struct AsyncResourceLoader<T: AsyncNew<T>> {
    pub(crate) task: Task<T>,
}

impl<T: AsyncNew<T> + Send + 'static> Default for AsyncResourceLoader<T> {
    fn default() -> Self {
        let thread_pool = AsyncComputeTaskPool::get();
        let task = thread_pool.spawn(async {
            static MODULE_PREFIX: LazyLock<Regex> =
                LazyLock::new(|| Regex::new(r"[^:<>]+::").expect("Valid regex for module prefix"));

            let start = Instant::now();
            let type_name = MODULE_PREFIX.replace_all(type_name::<T>(), "");
            println!("Started loading {type_name}");

            let result = T::async_new().await;

            let duration = start.elapsed();
            println!("Finished loading {type_name} in {duration:?}");

            result
        });
        Self { task }
    }
}

fn create_async_resource<T: AsyncNew<T> + Resource>(
    mut commands: Commands,
    mut async_resource_generator: ResMut<AsyncResourceLoader<T>>,
) {
    if let Some(async_resource) = block_on(poll_once(&mut async_resource_generator.task)) {
        commands.insert_resource(async_resource);
    }
}