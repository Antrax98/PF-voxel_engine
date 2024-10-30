use bevy::{math::UVec3, prelude::Component, utils::hashbrown::HashMap};

//* usar u8 como codigo o mascara para determinar el color del voxel */
pub struct Brickmap {
    solid_mask : [u8;512]
}
impl Brickmap {
    fn new() -> Self {
        Brickmap{
            solid_mask : [0;512]
        }
    }
}

//aumentar la cantidad de datos de ser necesario
#[derive(Default,Debug)]
pub struct VoxelData {
    color: u8
}
impl VoxelData {
    fn default() -> Self {
        VoxelData{
            color: 0
        }
    }
}

#[derive(Default,Debug)]
pub struct BrickMap {
    voxel_mask : [u64; 8],
    voxel_data : Vec<VoxelData>
}
impl BrickMap {
    pub fn default() -> Self {
        BrickMap {
            voxel_mask: [0b00000000_00000000_00000000_00000000;8],// 8x8x8=512, u64=64=8x8, 8xu64=512
            voxel_data: Vec::new()
        }
    }
    //TODO: implementar funciones que modifiquen el Brickmap de forma segura (preferiblemente con "bitwise operators")
}


//TODO: quitar Option<Box<T>> del Brickmap para limitar un poco el uso de punteros???
//?: implemetar chunk como un Entity y guardar su id en el chunk_hash de VoxelWorld.
//TODOR: implemetar forma de detectar cuando el chunk deve ser comprimido/descartado.
//TODO: al momento de spawnear el chunk, hacerlo junto con otro componente que guarde su ubicacion 3d (vec3) ¿o todo en el mismo struct?
pub struct Chunk {
    brickmap_array: Option<Box<[Option<Box<Brickmap>>; 4096]>>,
    //primer optionBox, puede que el chunk completo este vacio
    //segundo optionBox, puede que brickmaps unitarios enten vacios (probablemente qutarlo)
    state: ChunkState,
    version: u64
}

pub enum ChunkState {
    Loaded, //si el chunk esta totalmente cargado en memoria //? guardar los datos del chunk
    Loading,//si el chunk esta siendo deserealisado y descomprimido o generado de forma procedural //? guardar Promesa? de donde se obtendran los datos
    Unloading,//si el chunk se esta serialisando y comprimiendo para guardar en disco //? guardar promesa? que al terminar podra ser eliminado este chunk
    Sheduled //si el chunk fue recien creado y en espera que la funcion de carga lo tome //?Probablemente no se use?(pasaria a un estado de loading al ser creado)
}


// IDEA: en vez de Box<Chunk>>, crear entidades con componentes Chunk y agregar EntityID al Hash
// IDEA 2: desechar la ide de un hash y solo usar los componentes chunk por si solos (pero añadir algo para destinguir si pertenecen al mismo mundo)
#[derive(Component)]
pub struct VoxelWorld {
    pub chunk_hash : HashMap<UVec3,Box<Chunk>>
}

