use voxel_engine::*;
use voxel_render::*;
use bevy::prelude::*;
use iyes_perf_ui::prelude::*;

const SIZE: (u32,u32) = (512,512);

fn main() {

    println!("Hello, world!");
    //TODO: relegar la creacion de la pantalla a VOXEL_RENDER
    App::new()
    .insert_resource(ClearColor(Color::BLACK))
    //Tamaño de 512x512 solo para ser constantes en las puebas
    .add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution : (
                        (SIZE.0) as f32,
                        (SIZE.1) as f32,
                    ).into(),
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    ))
    .add_plugins(VoxelEnginePlugin)
    .add_plugins(VoxelRenderPlugin)

    //Mostrar el framerate
    .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
    //.add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
    //.add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
    .add_plugins(PerfUiPlugin)
    .add_systems(Startup, perf_ui_setup)

    .run();

}

fn perf_ui_setup(
    mut commands: Commands
){
    commands.spawn(PerfUiCompleteBundle::default());
}