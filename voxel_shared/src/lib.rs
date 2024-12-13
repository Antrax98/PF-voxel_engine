use bevy::{math::{Mat4, UVec3}, prelude::{Component, Event}, render::render_resource::{ShaderSize, ShaderType}, utils::hashbrown::HashMap};
use bitvec::prelude::*;
use bytemuck::{AnyBitPattern, NoUninit, Pod, Zeroable};


/* 
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
*/



// //aumentar la cantidad de datos de ser necesario
// #[derive(Default,Debug)]
// pub struct VoxelData {
//     color: u8
// }
// impl VoxelData {
//     fn default() -> Self {
//         VoxelData{
//             color: 0
//         }
//     }
// }

//usarlo dentro del Chunk
#[repr(C)]
#[derive(Debug,Clone, Copy)]
pub struct BrickMap {
    pub datos: BitArray<[u32; 16], Msb0> //(u32)2x8 -> 8x8x8
}

impl Default for BrickMap {
    fn default() -> Self {
        BrickMap{
            datos: bitarr![u32, Msb0;0;512]
        }
    }


}

//TODO: quitar Option<Box<T>> del Brickmap para limitar un poco el uso de punteros???
//?: implemetar chunk como un Entity y guardar su id en el chunk_hash de VoxelWorld.
//TODOR: implemetar forma de detectar cuando el chunk deve ser comprimido/descartado.
//TODO: al momento de spawnear el chunk, hacerlo junto con otro componente que guarde su ubicacion 3d (vec3) ¿o todo en el mismo struct?
pub struct Chunk {
    //USARLO ESTRICTAMERNTE CON TAMAÑO 4096
    pub brickmap_array: Option<Vec<BrickMap>>,
    //primer optionBox, depende del ChunkState
    pub state: ChunkState,
    pub version: u64
}
impl Chunk {
    pub fn loading() -> Self {
        Chunk{
            brickmap_array: None,
            state: ChunkState::Loading,
            version: 0
        }
    }
}

pub enum ChunkState {
    Loaded, //si el chunk esta totalmente cargado en memoria //? guardar los datos del chunk
    Loading,//si el chunk esta siendo deserealisado y descomprimido o generado de forma procedural //? guardar Promesa? de donde se obtendran los datos
    Unloading,//si el chunk se esta serialisando y comprimiendo para guardar en disco //? guardar promesa? que al terminar podra ser eliminado este chunk
}


// IDEA: en vez de Box<Chunk>>, crear entidades con componentes Chunk y agregar EntityID al Hash
// IDEA 2: desechar la ide de un hash y solo usar los componentes chunk por si solos (pero añadir algo para destinguir si pertenecen al mismo mundo)
#[derive(Component)]
pub struct VoxelWorld {
    pub chunk_hash : HashMap<UVec3,Chunk>
}


//Mayormente Renderer

//SOLO usar vec3 o similares si el struct no sera reenviado al cpu y solo se leera en gpu
#[repr(C)]
#[derive(ShaderType,Debug,AnyBitPattern, NoUninit,Clone, Copy)]
pub struct InitData{
    pub imagen_height : u32,
    pub imagen_width : u32,
    pub feedback_buffer_size : u32,
    pub world_size_x : u32,
    pub world_size_y : u32,
    pub world_size_z : u32,
}


#[repr(C)]
#[derive(ShaderType,Debug,AnyBitPattern, NoUninit,Clone, Copy)]
pub struct NeoVec3{
    pub x: f32,
    pub y: f32,
    pub z: f32
}
impl Default for NeoVec3{
    fn default() -> Self {
        NeoVec3{
            x: 0.,
            y: 0.,
            z: 0.
        }
    }
}
impl NeoVec3 {
    pub fn forward() -> Self {
        NeoVec3{
            x: 0.,
            y: 0.,
            z: -1.
        }
    }
    pub fn nuevo(x:f32, y:f32, z:f32) -> Self {
        NeoVec3{
            x,
            y,
            z
        }
    }
}



#[repr(C)]
#[derive(ShaderType,Debug,AnyBitPattern, NoUninit,Clone, Copy)]
pub struct NeoVec4{
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32
}
impl Default for NeoVec4{
    fn default() -> Self {
        NeoVec4{
            x: 0.,
            y: 0.,
            z: 0.,
            w: 0.
        }
    }
}
impl NeoVec4 {
    pub fn nuevo(x:f32, y:f32, z:f32, w:f32) -> Self {
        NeoVec4{
            x,
            y,
            z,
            w
        }
    }

    pub fn from_vec4(aux: &[f32;4]) -> Self {
        NeoVec4{x:aux[0],y:aux[1],z:aux[2],w:aux[3]}
    }
}

#[repr(C)]
#[derive(ShaderType,Debug,AnyBitPattern, NoUninit,Clone, Copy)]
pub struct NeoUVec3{
    pub x: u32,
    pub y: u32,
    pub z: u32
}
impl Default for NeoUVec3{
    fn default() -> Self {
        NeoUVec3{
            x: 0,
            y: 0,
            z: 0
        }
    }
}

#[repr(C)]
#[derive(ShaderType,Debug,AnyBitPattern, NoUninit,Clone, Copy)]
pub struct NeoUVec4{
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub w: u32
}
impl Default for NeoUVec4{
    fn default() -> Self {
        NeoUVec4{
            x: 0,
            y: 0,
            z: 0,
            w: 0
        }
    }
}

#[repr(C)]
#[derive(ShaderType,Debug,AnyBitPattern, NoUninit,Clone, Copy)]
pub struct NeoMat4{
    pub x_axis: NeoVec4,
    pub y_axis: NeoVec4,
    pub z_axis: NeoVec4,
    pub w_axis: NeoVec4,
}
impl NeoMat4 {
    pub fn IDENTITY() -> Self {
        NeoMat4{
            x_axis: NeoVec4{x: 1., y: 0., z: 0., w: 0.},
            y_axis: NeoVec4{x: 0., y: 1., z: 0., w: 0.},
            z_axis: NeoVec4{x: 0., y: 0., z: 1., w: 0.},
            w_axis: NeoVec4{x: 0., y: 0., z: 0., w: 1.},
        }
    }
    pub fn from_mat4(aux : Mat4) -> Self{
        NeoMat4 {
            x_axis: NeoVec4::from_vec4(&aux.x_axis.to_array()),
            y_axis: NeoVec4::from_vec4(&aux.y_axis.to_array()),
            z_axis: NeoVec4::from_vec4(&aux.z_axis.to_array()),
            w_axis: NeoVec4::from_vec4(&aux.w_axis.to_array()) 
        }
    }
}



#[repr(C)]
#[derive(ShaderType,Debug,AnyBitPattern, NoUninit,Clone, Copy)]
pub struct VarData{
    pub source : NeoVec3,//donde esta la camara en el mundo
    pub direction: NeoVec3,//direccion a la que esta mirando la camara
    pub fov: f32,//field of view, por ahora es f32
    pub camera_mat: NeoMat4,
    pub time: u32,
    pub feedback_idx: u32, //atomico encargado de dictar a que indice del feedback_buffer escribir y leer;
    pub command_buffer_size: u32
}
impl Default for VarData{
    fn default() -> Self {
        VarData{
            source: NeoVec3::nuevo(0., 0., 0.),
            direction: NeoVec3::nuevo(0., 0., 0.),
            fov: 80.,
            camera_mat: NeoMat4::IDENTITY(),
            time: 0,
            feedback_idx: 0,
            command_buffer_size: 0
        }
    }
}

//SOLO USAR PARA ENVIARLO AL SHADER
#[repr(C)]
#[derive(ShaderType,Debug,Pod,Zeroable,Clone, Copy)]
pub struct Brickmap {
    pub datos: [u32;16] //(u32)2x8 -> 8x8x8
}

impl Default for Brickmap {
    fn default() -> Self {
        Brickmap{
            datos: [0 as u32;16]
        }
    }
}
impl Brickmap {
    pub fn cpu_to_gpu(aux: BrickMap) -> Brickmap {
        Brickmap { datos: aux.datos.data }
    }
}



#[derive(Event,Debug)]
pub struct CellsRequest(pub Vec<NeoUVec3>);

#[derive(Event,Debug)]
pub struct CellsResponse(pub Vec<(Option<BrickMap>,u32)>);


//tamaño en brickmaps
//multiplos de 16 para que coincida con los chunks
//TODO: obtener el valor al iniciar el ejecutable
pub const WORLD_SIZE: (u32,u32,u32)= (256,256,256);