#![forbid(absolute_paths_not_starting_with_crate)]
#![forbid(ambiguous_associated_items)]
#![forbid(anonymous_parameters)]
#![forbid(array_into_iter)]
#![forbid(bare_trait_objects)]
#![forbid(cenum_impl_drop_cast)]
#![forbid(coherence_leak_check)]
#![forbid(conflicting_repr_hints)]
#![forbid(const_err)]
#![forbid(const_evaluatable_unchecked)]
#![forbid(deref_into_dyn_supertrait)]
#![forbid(ellipsis_inclusive_range_patterns)]
#![forbid(explicit_outlives_requirements)]
#![forbid(forbidden_lint_groups)]
#![forbid(ill_formed_attribute_input)]
#![forbid(illegal_floating_point_literal_pattern)]
#![forbid(indirect_structural_match)]
#![forbid(invalid_doc_attributes)]
#![forbid(invalid_type_param_default)]
#![forbid(keyword_idents)]
#![forbid(late_bound_lifetime_arguments)]
#![forbid(legacy_derive_helpers)]
#![forbid(macro_expanded_macro_exports_accessed_by_absolute_paths)]
#![forbid(missing_fragment_specifier)]
#![forbid(non_fmt_panics)]
#![forbid(non_snake_case)]
#![forbid(nontrivial_structural_match)]
#![forbid(order_dependent_trait_objects)]
#![forbid(patterns_in_fns_without_body)]
#![forbid(pointer_structural_match)]
#![forbid(private_in_public)]
#![forbid(proc_macro_back_compat)]
#![forbid(proc_macro_derive_resolution_fallback)]
#![forbid(pub_use_of_private_extern_crate)]
#![forbid(rust_2021_incompatible_closure_captures)]
#![forbid(rust_2021_incompatible_or_patterns)]
#![forbid(rust_2021_prefixes_incompatible_syntax)]
#![forbid(rust_2021_prelude_collisions)]
#![forbid(semicolon_in_expressions_from_macros)]
#![forbid(soft_unstable)]
#![forbid(tyvar_behind_raw_pointer)]
#![forbid(unaligned_references)]
#![forbid(uninhabited_static)]
#![forbid(unstable_name_collisions)]
#![forbid(unsupported_calling_conventions)]
#![forbid(where_clauses_object_safety)]
#![deny(unused)]
#![warn(dead_code)]
#![deny(non_camel_case_types)]
#![deny(non_upper_case_globals)]
#![deny(unreachable_code)]
#![deny(unreachable_pub)]
#![deny(unused_attributes)]
#![deny(unused_extern_crates)]
#![deny(unused_macros)]
#![deny(clippy::complexity)]
#![deny(clippy::correctness)]
#![deny(clippy::pedantic)]
#![deny(clippy::perf)]
#![deny(clippy::style)]
#![deny(clippy::suspicious)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::match_wildcard_for_single_variants)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::option_if_let_else)] // worse readability
#![allow(clippy::redundant_else)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_arguments)] // doesn't work well with bevy
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::type_complexity)] // doesn't work well with bevy
#![allow(clippy::only_used_in_recursion)] // false positives

mod cdda;
mod components;
mod core;
mod plugin;
mod prelude;
mod resources;
mod systems;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::{App, DefaultPlugins};
use plugin::RustaclysmPlugin;

fn main() {
    App::new()
        .add_plugin(RustaclysmPlugin) // first, to prevent vulkan errors
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .run();
}
