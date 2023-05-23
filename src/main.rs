use bevy::prelude::*;
use bevy_spine::{prelude::*, SpineDrawer};

fn main() {
    //asset_exporter::export().unwrap();
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SpinePlugin)
        .add_plugin(SpinePbrPlugin)
        .add_plugin(CursorPlugin)
        .add_startup_system(setup)
        .add_system(on_ready.in_set(SpineSet::OnReady))
        .run();
}

fn setup(
    mut commands: Commands,
    mut skeletons: ResMut<Assets<SkeletonData>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    let skeleton: Handle<SkeletonData> = skeletons.add(SkeletonData::new_from_json(
        asset_server.load("pbr_showcase/pbr_showcase.json"),
        asset_server.load("pbr_showcase/pbr_showcase.atlas"),
    ));
    commands.spawn((
        SpineBundle {
            skeleton,
            settings: SpineSettings {
                default_materials: false,
                drawer: SpineDrawer::Separated,
                ..Default::default()
            },
            ..Default::default()
        },
        SpinePbr {
            normal_map: asset_server.load("pbr_showcase/pbr_showcase_n.png"),
        },
    ));
}

fn on_ready(
    mut spine_ready_events: EventReader<SpineReadyEvent>,
    mut spine_query: Query<&mut Spine, With<SpinePbr>>,
) {
    for spine_ready_event in spine_ready_events.iter() {
        if let Ok(mut spine) = spine_query.get_mut(spine_ready_event.entity) {
            let _ = spine
                .animation_state
                .set_animation_by_name(0, "animation", true);
        }
    }
}

mod cursor;
mod spine_pbr;

use cursor::*;
use spine_pbr::*;
