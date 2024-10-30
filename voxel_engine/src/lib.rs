use bevy::{
    prelude::*,
    utils::{hashbrown::HashMap},
};
use voxel_shared::*;


pub struct VoxelEnginePlugin;

impl Plugin for VoxelEnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, voxel_init);
    }
}



pub fn voxel_init(mut commands: Commands) {

    //Aqui crear el voxel world como una entidad y crear un struct que le de nombre de MainWorld o mundo 1 o algo asi
    println!("inicialisando mundo de voxeles");

    commands.spawn(
        VoxelWorld {
            chunk_hash: HashMap::new()
        }
    );

    println!("Se ha inicialisado el mundo de voxeles");
}


