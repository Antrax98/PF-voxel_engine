use std::cell;

use bevy::{
    prelude::*, tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task}, utils::hashbrown::HashMap
};
use voxel_shared::*;
use bitvec::prelude::*;

use noise::{NoiseFn, Perlin, Seedable};

const SEED: u32 = 1;

pub struct VoxelEnginePlugin;

impl Plugin for VoxelEnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, voxel_world_init);



        app.add_systems(Update, feedback_chunkgen_system);
        app.add_systems(Update, chunk_taskpool_receiver);



        app.insert_resource(ChunkGenerationTasks{
            chunkgen_tasks: HashMap::new()
        });

        app.add_event::<CellsResponse>();
    }
}



pub fn voxel_world_init(mut commands: Commands) {

    //Aqui crear el voxel world como una entidad y crear un struct que le de nombre de MainWorld o mundo 1 o algo asi
    println!("inicialisando mundo de voxeles");

    commands.spawn(
        VoxelWorld {
            chunk_hash: HashMap::new()
        }
    );

    println!("Se ha inicialisado el mundo de voxeles");
}


#[derive(Resource)]
pub struct ChunkGenerationTasks {
    pub chunkgen_tasks: HashMap<UVec3, Task<Vec<BrickMap>>>
}

//type bm_container = BitArr!(for 40, in u64, Msb0);



fn feedback_chunkgen_system(
    mut ev_cellrequest: EventReader<CellsRequest>,
    mut voxel_world: Query<&mut VoxelWorld>,
    mut chunk_gen_task: ResMut<ChunkGenerationTasks>,
    mut ev_cellsender: EventWriter<CellsResponse>
) {
    let mut vw = &mut voxel_world.single_mut().chunk_hash;
    
    let task_pool = AsyncComputeTaskPool::get();
    let mut brickmap_response: Vec<(BrickMap,u32)> = vec![];
    let mut brickmap_to_send: Vec<i32>;
    let mut chunk_to_gen: Vec<UVec3>= vec![];
    //ciclo for que, de ya estar cargado el chunk, lo añade a un vector que se enviara de vuelta al voxel-renderer, de no estar vacio dicho vector
    //de no estar cargado, se añadira el chunk al que pertenece a otro vector vec<vec3>
    for ev in ev_cellrequest.read() {
        for cell_coord in &ev.0{
            //Coordenadas del chunk que contiene esta cell/brickmap
            let chunk_coord: UVec3 = UVec3::new(cell_coord.x/16,cell_coord.y/16,cell_coord.z/16);
            if vw.contains_key(&chunk_coord){
                let chunk = vw.get(&chunk_coord).unwrap();
                match chunk.state {
                    ChunkState::Loaded => {
                        // agregar brickMap al vector de respuesta
                        //TODO: PUEDE QUE ESTA PARTE CONTENGA ERRORES (SOLO PROBABLE devido a la conversion entre cordenadas brickmap y cordenadas globales)
                        brickmap_response.push((chunk.brickmap_array.as_ref().unwrap()[(((((cell_coord.x%16)*16)+(cell_coord.y%16))*16)+(cell_coord.z%16))as usize].clone(),(((((cell_coord.x)*16)+(cell_coord.y))*16)+(cell_coord.z))));
                    },
                    ChunkState::Loading => println!("Loading Chunk => {}",chunk_coord),
                    ChunkState::Unloading => println!("Unloading Chunk => {}",chunk_coord)
                }
            }else {
                //aqui enviar los chunk_gen_tasks
                if chunk_gen_task.chunkgen_tasks.contains_key(&chunk_coord){
                    //aqui si el task para este chunk ya esxiste
                }else{
                    //aqui si el task NO EXOSTE
                    let task = task_pool.spawn(async move {
                        let voxel_world_size: UVec3 = UVec3::new(WORLD_SIZE.0,WORLD_SIZE.1,WORLD_SIZE.2) *16*8;
                        //AQUI GENERACION DEL CHUNK
                        let c_coord = chunk_coord.clone();
                        let n_map = Perlin::new(SEED);
                        let mut dat_box: Vec<BrickMap> = Vec::new();
                        for bm_idx in 0..4096 {
                            let bm_coords: UVec3 = UVec3 { x: (bm_idx/(256)), y: ((bm_idx/16)%16), z: (bm_idx%16) };
                            let mut bm_aux: BitArray<[u32; 16], Msb0> = bitarr![u32, Msb0;0;512];
                            for v_idx in 0..512 {
                                let v_coords: UVec3 = UVec3 { x: (v_idx/(64)), y: ((v_idx/8)%8), z: (v_idx%8) } % 16;
                                //AQUI CREAR EL BRICKMAP POR CADA VOXEL
                                let detalle = 0.001;
                                let voxel_val = n_map.get([(v_coords.x +(bm_coords.x * 8)+(c_coord.x*16*8)) as f64 * detalle,0 as f64,(v_coords.z +(bm_coords.z * 8)+(c_coord.z*16*8)) as f64]).abs()* detalle;
                                //let v_aux: UVec2 = UVec2 { x: v_coords.x +(bm_coords.x * 8)+(c_coord.x*16*8), y: () };

                                //value = funcion que devuelve true/false
                                bm_aux.set(v_idx as usize, {
                                    if (voxel_val*128.) as u32 == v_coords.z {
                                        true
                                    } else {
                                        false
                                    }
                                });

                                bm_aux.set(v_idx as usize, true);
                            }
                            //AQUI AGREGAR EL BRICKMAP AL DATBOX
                            dat_box.push(BrickMap { datos: bm_aux});
                        }




                        //retornar Chunk
                        ///CAMBIAR ESTA PARTE PARA QUE REGRESE VALORES REALES Y NO SOLO CEROS
                        //let dat_box= vec![BrickMap::default(); 4096];
                        dat_box
                    });
                    //inserta el task id en el chunkgen_task
                    chunk_gen_task.chunkgen_tasks.insert(chunk_coord, task);
                    //inserta el chunk con estado Loading en el voxel_world
                    vw.insert(chunk_coord, Chunk::loading());
                }
            }
        }

    }

    //Enviar los brickmaps al voxel_render
    if brickmap_response.len() != 0 {
        ev_cellsender.send(CellsResponse(brickmap_response));
    }
    
}

//guarda los chunks recividos en el voxel_world
fn chunk_taskpool_receiver(
    mut voxel_world: Query<&mut VoxelWorld>,
    mut chunk_gen_task: ResMut<ChunkGenerationTasks>
) {
    let mut vw = &mut voxel_world.single_mut().chunk_hash;

    chunk_gen_task.chunkgen_tasks.retain(|chunk_coord,task| {
        //chekear el estatus del task
        let status = block_on(future::poll_once(task));

        //mantiene el entry en el HashMap solo si el task aun no a terminado
        let retain = status.is_none();

        //si el task termino, hacer algo con los datos
        if let Some(mut chunk_data) = status {
            //GUARDAR CHUNKDATA EN EL VOXEL WORLD HASHMAP
            let mut chunk_mut = vw.get_mut(chunk_coord).unwrap();
            chunk_mut.brickmap_array = Some(chunk_data);
            chunk_mut.state = ChunkState::Loaded;
        }

        retain
    });


}


fn feedback_cell_sender(
    voxel_world: Query<&VoxelWorld>,
) {
    //LeER LOS CELLS que se necesitan, obtenerlos de los chunks que ya esten cargados
    //y enviarlos de forma (vec3,cell) -> hacerle un 
}