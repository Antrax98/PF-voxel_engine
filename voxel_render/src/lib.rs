use bevy::{
    a11y::accesskit::Size, prelude::*, render::{prelude::Image, render_asset::RenderAssetUsages, render_resource::*, texture}
};
use bytemuck::{AnyBitPattern, NoUninit, Pod, Zeroable};
use bitvec::prelude::*;
use std::{convert::TryInto, f32::consts::PI, iter};
use bevy_easy_compute::prelude::*;
use voxel_shared::*;

use bevy::app::AppExit;
use bevy::input::mouse::MouseMotion;

use bevy::window::{CursorGrabMode, PrimaryWindow};


const COMPUTE_SHADER_PATH: &str = "shaders/voxel_processor.wgsl";

//Tamaño de la ventana
//TODO: obtener el valor a travez del AddPlugins() Params
//preferiblemente usar multiplos de 8?
//en multiplos de 64 (devido a el tamaño del workgroup de la GPU)
const SIZE: (u32,u32) = (512,512);
//?a que se refiere con esto??????
const DISPLAY_FACTOR: u32 = 4;
const WORKGROUP_SIZE: u32 = 8;

const FEEDBACK_BUFFER_SIZE: usize = 10;

//tamaño en brickmaps
//TODO: obtener el valor al iniciar el ejecutable
const WORLD_SIZE: (u32,u32,u32)= (256,256,256);


pub struct VoxelRenderPlugin;

impl Plugin for VoxelRenderPlugin {
    fn build(&self, app: &mut App) {
        

        app.add_systems(Startup, instanciar_renderisadores);
        app.add_plugins(AppComputePlugin);
        app.add_plugins(AppComputeWorkerPlugin::<VoxelRenderWorker>::default());
        //app.add_systems(Update, test);
        app.add_systems(Update, actualizar_imagen);
        app.add_systems(Update, deteccion_movimiento);
        //app.add_systems(Update, movimiento_mouse);
        app.add_systems(Startup, cursor_grab);
        app.add_systems(Startup, camera3Dsetup);





    }
}

fn deteccion_movimiento(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut evr_motion: EventReader<MouseMotion>,
    mut exit: EventWriter<AppExit>,
    mut compute_worker: ResMut<AppComputeWorker::<VoxelRenderWorker>>,
    mut camera_t: Query<&mut Transform, (With<MovementCamera>)>,
) {
    if !compute_worker.ready() {
        return
    }

    let Ok(mut camera_t) = camera_t.get_single_mut() else {
        return;
    };

    let mut var_dat: VarData = compute_worker.try_read("variable_data").unwrap();
    


    //Aqui cambiar todos los datos necesarios
    let mut direccion: Vec3 = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        direccion += camera_t.forward().as_vec3();
    }
    if keys.pressed(KeyCode::KeyS) {
        direccion += camera_t.back().as_vec3();
    }
    if keys.pressed(KeyCode::KeyA) {
        direccion += camera_t.left().as_vec3();
    }
    if keys.pressed(KeyCode::KeyD) {
        direccion += camera_t.right().as_vec3();
    }
    let movimineto = direccion.normalize_or_zero() * time.delta_seconds();//multiplicar por un escalar para hacer el movimiento mas rapido
    camera_t.translation += movimineto;
    

    if keys.pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }


    //borrar esto??
    /* 
    let dir = camera_t.forward().as_vec3();
    var_dat.direction = NeoVec3::nuevo(dir.x,dir.y,dir.z);
    */
    
    for ev in evr_motion.read() {
        camera_t.rotate_y(-ev.delta.x * 0.03);
        camera_t.rotate_x(-ev.delta.y * 0.03);
        
    }
    let forward = camera_t.forward(); 


    //guardar datos en variable
    var_dat.direction = NeoVec3::nuevo(forward.x,forward.y,forward.z);
    var_dat.source = NeoVec3::nuevo(camera_t.translation.x,camera_t.translation.y,camera_t.translation.z);
    
    
    
    println!("source: {:?} + direcction: {:?}",var_dat.source,var_dat.direction);
    
    
    //escribir variables en bvuffer
    compute_worker.write("variable_data", &var_dat);



}

#[derive(Component)]
struct MovementCamera;


fn camera3Dsetup(
    mut commands: Commands
) {

    //no es necesario que sea una camara de verdad, solo un transform y la etiqueta MovementCamera
    commands.spawn((
        Camera3dBundle{
            camera: Camera{
                order: 1,
                ..default()
            },
            transform: Transform::from_xyz(50., 50., 10.)
                .looking_at(-Vec3::Z, Vec3::Y),
            
            camera_3d: Camera3d {
                ..default()
            },
            ..default()
        },
        MovementCamera
    ));


}

//al parecer, tener dos sistemas que lleen y escriben en un mismo buffer los contraresta?
fn movimiento_mouse(
    mut evr_motion: EventReader<MouseMotion>,
    mut compute_worker: ResMut<AppComputeWorker::<VoxelRenderWorker>>,
) {

    if !compute_worker.ready() {
        return
    }

    let mut var_dat: VarData = compute_worker.try_read("variable_data").unwrap();
    //let mut cam_vec: Vec<f32> = vec![var_dat.source.x,var_dat.source.y,var_dat.source.z];

    //let mut theta = var_dat.source.x.atan2(var_dat.source.z);

    //let mut thwta2 = 0;

    //let cs = theta.cos();
    //let sn = theta.sin();


    /* 
    for ev in evr_motion.read() {
        let theta = var_dat.direction.x.atan2(var_dat.direction.z) + 0.11;
        let cs = theta.cos();
        let sn = theta.sin();
        let newx = var_dat.direction.x * cs - var_dat.direction.z * sn;
        let newz = var_dat.direction.x * sn + var_dat.direction.z * cs;
        var_dat.direction.x = newx;
        var_dat.direction.z = newz;
        println!("{:?}", var_dat.direction);
    }
    */
    for ev in evr_motion.read() {
        var_dat.direction.x = var_dat.direction.x + ev.delta.x;
        var_dat.direction.z = var_dat.direction.z + ev.delta.y;
    }

    compute_worker.write("variable_data", &var_dat);

}

fn cursor_grab(
    mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut primary_window = q_windows.single_mut();

    primary_window.cursor.grab_mode = CursorGrabMode::Locked;

    //primary_window.cursor.visible = false;
}

#[derive(TypePath)]
struct VoxelRenderShader;
impl ComputeShader for VoxelRenderShader {
    fn shader() -> ShaderRef {
        COMPUTE_SHADER_PATH.into()
    }
}

//SOLO usar vec3 o similares si el struct no sera reenviado al cpu y solo se leera en gpu
#[repr(C)]
#[derive(ShaderType,Debug,AnyBitPattern,Clone, Copy)]
struct InitData{
    imagen_height : u32,
    imagen_width : u32
}


#[repr(C)]
#[derive(ShaderType,Debug,AnyBitPattern, NoUninit,Clone, Copy)]
struct NeoVec3{
    x: f32,
    y: f32,
    z: f32
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
    fn forward() -> Self {
        NeoVec3{
            x: 0.,
            y: 0.,
            z: -1.
        }
    }
    fn nuevo(x:f32, y:f32, z:f32) -> Self {
        NeoVec3{
            x,
            y,
            z
        }
    }
}

#[repr(C)]
#[derive(ShaderType,Debug,AnyBitPattern, NoUninit,Clone, Copy)]
struct NeoUVec3{
    x: u32,
    y: u32,
    z: u32
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
struct VarData{
    source : NeoVec3,//donde esta la camara en el mundo
    direction: NeoVec3,//direccion a la que esta mirando la camara
    fov: f32,//field of view, por ahora es f32
    used_buffer: u32, //atomico encargado de dictar a que indice del feedback_buffer escribir y leer;
    time: u32,
}

//TODO: mover a voxel shader para ser usado por voxel_engine
///!solo usar para pruebas
#[repr(C)]
#[derive(ShaderType,Debug,AnyBitPattern,Clone, Copy)]
struct Brickmap {
    datos: [u32;128] //(u32)2x8 -> 8x8x8
}

impl Default for Brickmap {
    fn default() -> Self {
        Brickmap{
            datos: [0 as u32;128]
        }
    }


}





//cambiar dependiendo de la vram de la gpu y el ratio brickmap-=-colorData
const BD_SIZE: usize = 1000;


#[derive(Resource)]
struct VoxelRenderWorker;
impl ComputeWorker for VoxelRenderWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        let init_data: InitData= InitData{
            imagen_height: SIZE.0/64,
            imagen_width: SIZE.1/64,
        };

        let mut var_data: VarData= VarData{
            source: NeoVec3::nuevo(50.,50.,10.),//Vec3 { x: 0., y: 0., z: 0. },
            direction: NeoVec3::nuevo(0.,0.,1.),//Vec3 { x: 0., y: 0., z: -1. },
            fov: 50.,
            used_buffer: 0,
            time: 0
        };
        var_data.source.x = 50.;
        var_data.source.y = 50.;
        var_data.source.z = 50.; 

        let blank_image = [0 as u32;(SIZE.0 as usize)*(SIZE.1 as usize)];


        const ws :u32 = WORLD_SIZE.0*WORLD_SIZE.1*WORLD_SIZE.2;

        //byte 1 2 y 3 = Rgb LOD, byte 4 = flags
        //flags 00000000 = empty
        const def_brickgrid_cell: u32= 0xff0000ff;//0b00000000_00000000_00000000_00000000

        #[repr(C)]
        #[derive(ShaderType,Debug,AnyBitPattern,Clone, Copy)]
        struct ato {
            at: u32
        }

        //let gri:Vec<u32> = test_brickgrid(WORLD_SIZE, ws);
        // [def_brickgrid_cell;ws as usize]
        //let test_grid = &gri[..16777216];

        let worker = AppComputeWorkerBuilder::new(world)
        .add_uniform("data_struct",&init_data)
        .add_staging("variable_data", &var_data)
        .add_staging("imagen", &blank_image)
        .add_rw_storage("brickgrid", &[def_brickgrid_cell;ws as usize])
        .add_staging("brickmap_data", &[Brickmap::default();BD_SIZE])
        .add_rw_storage("feedback", &[NeoUVec3::default();FEEDBACK_BUFFER_SIZE])
        .add_pass::<VoxelRenderShader>([512,512,1],&["data_struct","variable_data","imagen","brickgrid","brickmap_data","feedback"])
        .build();

        worker
    }
    // LOS size TIENE QUE SER MULTIPLOS DE 4
}

fn test_brickgrid(world : (u32,u32,u32), ws: u32) -> Vec<u32> {

    //let mut grid: [u32;16777216] = [0x00000000;16777216];

    let mut grid: Vec<u32> = vec![];

    for i in 0..ws {
        println!("sdasd");
    }

    for z in 0..world.0 {
        for y in 0..world.1 {
            for x in 0..world.2 {
                if(z==50 && 10<y && y<120 && 10<x && x<120) {
                    //grid[(x + world.1 * (y +(world.2 * z))) as usize] = 0xff0000ff;
                    grid.push(0xff0000ff);
                }else{
                    grid.push(0x00000000);
                }

                
            }
        }
    }




    grid
}


fn test (
    mut compute_worker: ResMut<AppComputeWorker::<VoxelRenderWorker>>,
    time: Res<Time>,
){
    if !compute_worker.ready() {
        return
    }

    let tim = time.elapsed_seconds() as u32;

    let mut var_dat: VarData = compute_worker.try_read("variable_data").unwrap();
    
    //Aqui cambiar todos los datos necesarios
    var_dat.time = tim;

    compute_worker.write("variable_data", &var_dat);
}

fn read_feedback(
    compute_worker: ResMut<AppComputeWorker::<VoxelRenderWorker>>,
) {
    if !compute_worker.ready() {
        return
    }

    //TODO?; crear recurso en donde guardar el struct VarData
    // y crear systema que se ejecute al final del todo que lea el recurso y lo escriba en el buffer
    ////! de no hacerlo podria causar que se pierda informacion entre los systemas que usan este struct

    let vardat: VarData = compute_worker.read("variable_data");
    let feedback: Vec<NeoUVec3> = compute_worker.read_vec("feedback");










    //Aqui ver como pedir y recivir brickmaps para despues enviarlos



}

fn actualizar_imagen(
    compute_worker: ResMut<AppComputeWorker::<VoxelRenderWorker>>,
    mut images: ResMut<Assets<Image>>,
    image_res: ResMut<RenderedImage>
) {

    if !compute_worker.ready() {
        return
    }
    

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

    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            ..default()
        },
        ..default()
    });

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

fn brickgrid_test_creation(world_size: usize) -> (Vec<u32>,Vec<Brickmap>) {
    let mut resultado: (Vec<u32>,Vec<Brickmap>) = (Vec::with_capacity(world_size),Vec::with_capacity(world_size));
    
    resultado.0 = vec![0x00000000 as u32;world_size];
    resultado.1 = vec![Brickmap::default();world_size];









    resultado
}
