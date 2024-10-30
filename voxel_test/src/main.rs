use voxel_engine::*;
use voxel_render::*;
use bevy::prelude::*;

const SIZE: (u32,u32) = (512,512);

fn main() {

    println!("Hello, world!");

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
    .run();

}

