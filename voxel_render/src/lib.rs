use bevy::{
    a11y::accesskit::Size, prelude::*, render::{prelude::Image, render_asset::RenderAssetUsages, render_resource::*, texture}
};
use bytemuck::AnyBitPattern;
use bitvec::prelude::*;
use std::{convert::TryInto, iter};
use bevy_easy_compute::prelude::*;
use voxel_shared::*;


const COMPUTE_SHADER_PATH: &str = "shaders/voxel_processor.wgsl";

//Tamaño de la ventana
//TODO: obtener el valor a travez del AddPlugins() Params
//preferiblemente usar multiplos de 8?
//en multiplos de 64 (devido a el tamaño del workgroup de la GPU)
const SIZE: (u32,u32) = (512,512);
//?a que se refiere con esto??????
const DISPLAY_FACTOR: u32 = 4;
const WORKGROUP_SIZE: u32 = 8;

//tamaño en brickmaps
//TODO: obtener el valor al iniciar el ejecutable
const WORLD_SIZE: (u32,u32,u32)= (256,256,256);


pub struct VoxelRenderPlugin;

impl Plugin for VoxelRenderPlugin {
    fn build(&self, app: &mut App) {
        

        app.add_systems(Startup, instanciar_renderisadores);
        app.add_plugins(AppComputePlugin);
        app.add_plugins(AppComputeWorkerPlugin::<VoxelRenderWorker>::default());
        app.add_systems(Update, test);





    }
}


#[derive(TypePath)]
struct VoxelRenderShader;
impl ComputeShader for VoxelRenderShader {
    fn shader() -> ShaderRef {
        COMPUTE_SHADER_PATH.into()
    }
}

#[repr(C)]
#[derive(ShaderType,Debug,AnyBitPattern,Clone, Copy)]
struct InitData{
    imagen_height : u32,
    imagen_width : u32
}

//TODO: mover a voxel shader para ser usado por voxel_engine
///!solo usar para pruebas
#[repr(C)]
#[derive(ShaderType,Debug,AnyBitPattern,Clone, Copy)]
struct Brickmap {
    datos: u32
}

//cambiar dependiendo de la vram de la gpu
const BD_SIZE: usize = 1000;


#[derive(Resource)]
struct VoxelRenderWorker;
impl ComputeWorker for VoxelRenderWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        let init_data: InitData= InitData{
            imagen_height: SIZE.0/64,
            imagen_width: SIZE.1/64
        };

        const ws :u32 = WORLD_SIZE.0*WORLD_SIZE.1*WORLD_SIZE.2;

        //byte 1 2 y 3 = Rgb LOD, byte 4 = flags
        //flags 00000000 = empty
        const def_brickgrid_cell: u32= 0x00000000;//0b00000000_00000000_00000000_00000000

        let worker = AppComputeWorkerBuilder::new(world)
        .add_uniform("data_struct",&init_data)
        .add_staging("variable_data", &[(0 as u32)])
        .add_staging("imagen", &[0 as u32;(SIZE.0 as usize)*(SIZE.1 as usize)])
        .add_rw_storage("brickgrid", &[def_brickgrid_cell;ws as usize])
        .add_staging("brickmap_data", &[Brickmap{datos:12};BD_SIZE])
        .add_pass::<VoxelRenderShader>([512,512,1],&["data_struct","variable_data","imagen","brickgrid","brickmap_data"])
        .build();

        worker
    }
    // LOS size TIENE QUE SER MULTIPLOS DE 4
}

fn test(
    mut compute_worker: ResMut<AppComputeWorker::<VoxelRenderWorker>>,
    mut images: ResMut<Assets<Image>>,
    time: Res<Time>,
    mut image_res: ResMut<RenderedImage>
) {

    if !compute_worker.ready() {
        return
    }

    let tim = time.elapsed_seconds() as u32;

    compute_worker.write_slice("variable_data", &[tim]);
    
    //println!("{}",tim);

    let resu:Vec<u32> = compute_worker.read_vec ("imagen");


    //*se obtienen el handle dentro del recurso y se le entrega este */
    //*handle dentro de un getmut al assets para obtener una referencia mutable a esta imagen */

    //println!("Directo de la GPU: {}",resu.len());
    

    let mano = vec32_to_imagedata(&resu);
    



    let imag_handle = image_res.textura.clone();

    let im = images.get_mut(&imag_handle).unwrap();

    //println!("len of image {:?}",im.data.len());

    //println!("len of data after convercion {:?}",mano.len());

    im.data = mano;

}

// USAR Rgba8Unorm COMO FORMATO DE LA IMAGEN


//AQUI EMPIEZA LA CREACION DE LA IMAGEN VACIA

fn instanciar_renderisadores (
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
){
    
    let pixel_data: [u8;4] = [0,255,100,200];

    let new_image: Image = Image::new_fill(
        Extent3d{width:SIZE.0,height:SIZE.1,depth_or_array_layers:1},
        TextureDimension::D2,
        &pixel_data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD
    );


    let image_saved = images.add(new_image);


    commands.insert_resource(RenderedImage {
        textura: image_saved.clone()
    });

    commands.spawn(SpriteBundle{
        sprite: Sprite {
            custom_size: Some(Vec2::new(SIZE.0 as f32,SIZE.1 as f32)),
            ..default()
        },
        texture: image_saved,
        ..default()
    });

    commands.spawn(Camera2dBundle::default());

}


#[derive(Resource,Clone)]
struct RenderedImage{
    textura: Handle<Image>
}

//TODO: funcion que transforme Vec<u32> en Vec<u8>*4 mediante bits
//Necesario para leer buffer imagen y pasar sus datos a una textuera
//el buffer solo trabaja con u32 y los datos de una imagen estan en u8

fn vec32_to_imagedata(v:&Vec<u32>) -> Vec<u8>{
    let mut resultado: Vec<u8> = Vec::new();


    for val in v {
        let uper_half= (val >> 16) as u16;
        let lower_half= (val & 0xFFFF) as u16;
        
        //println!("{:08b}+{:08b}",uper_half,lower_half);
        
        /* 
        let first = (uper_half >> 8) as u8;
        let second = (uper_half &0xFF) as u8;
        let third = (lower_half >> 8) as u8;
        let fourth = (lower_half &0xFF) as u8;
        println!("{:08b}+{:08b}+{:08b}+{:08b}",first,second,third,fourth);
        */

        resultado.push((uper_half >> 8) as u8);
        resultado.push((uper_half & 0xFF) as u8);
        resultado.push((lower_half >> 8) as u8);
        resultado.push((lower_half & 0xFF) as u8);
        //println!("{:?}",resultado)//esto mata el programa
    }




    resultado
}


