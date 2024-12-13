use bevy::{
    a11y::accesskit::Size, prelude::*, reflect::TypeData, render::{prelude::Image, render_asset::RenderAssetUsages, render_resource::*, texture}
};
use bytemuck::{AnyBitPattern, NoUninit, Pod, Zeroable};
use bitvec::prelude::*;
use std::{cell::Cell, cmp::min, convert::TryInto, f32::consts::PI, iter};
use bevy_easy_compute::prelude::*;
use voxel_shared::*;

use bevy::app::AppExit;
use bevy::input::mouse::MouseMotion;

use bevy::window::{CursorGrabMode, PrimaryWindow};


const COMPUTE_SHADER_PATH: &str = "shaders/voxel_processor.wgsl";
const ALLOCATOR_SHADER_PATH: &str = "shaders/voxel_allocator.wgsl";

//Tamaño de la ventana
//TODO: obtener el valor a travez del AddPlugins() Params
//preferiblemente usar multiplos de 8?
//en multiplos de 64 (devido a el tamaño del workgroup de la GPU)
const SIZE: (u32,u32) = (512,512);
//?a que se refiere con esto??????
const DISPLAY_FACTOR: u32 = 4;
const WORKGROUP_SIZE: u32 = 8;

const FEEDBACK_BUFFER_SIZE: usize = 256;

const OBJECT_POOL_MAX_SIZE: usize = 100000; //aprox 1GB


pub struct VoxelRenderPlugin;

impl Plugin for VoxelRenderPlugin {
    fn build(&self, app: &mut App) {
        

        app.add_systems(Startup, instanciar_renderisadores);
        app.add_systems(Startup, cursor_grab);
        app.add_systems(Startup, camera3Dsetup);
        app.add_plugins(AppComputePlugin);
        app.add_plugins(AppComputeWorkerPlugin::<VoxelRenderWorker>::default());

        
        app.add_systems(Update, (buffer_to_resource,resource_to_buffer));
        
        app.add_systems(
            Update, 
            (
                actualizar_imagen,
                deteccion_movimiento,
                feedback_sender_system,
            )
            .after(buffer_to_resource)
            .before(resource_to_buffer)
            );
        
        app.add_systems(Update, (cell_receiver_allocator).before(feedback_sender_system));
        
        app.insert_resource(VariableData{data:VarData::default()});
        let feed_data_aux : Vec<NeoUVec3> = vec![NeoUVec3::default();FEEDBACK_BUFFER_SIZE];
        app.insert_resource(Feedback{data: feed_data_aux});
        app.insert_resource(ObjectPool{data: Vec::new(), indice: 0});
        app.insert_resource(CommandBuffer{comandos: Vec::new()});


        app.add_event::<CellsRequest>();



    }
}

#[derive(Resource)]
struct VariableData {
    data: VarData
}

#[derive(Resource)]
struct Feedback {
    data: Vec<NeoUVec3>
}

//mover a voxel_shared


fn feedback_sender_system(
    mut variable_data: ResMut<VariableData>,
    feedback_buffer : Res<Feedback>,
    compute_worker: ResMut<AppComputeWorker::<VoxelRenderWorker>>,
    mut ev_cellrequest: EventWriter<CellsRequest>
) {

    if !compute_worker.ready() {
        return
    }

    // let mut var_dat: &mut VarData = &mut variable_data.data;
    // let mut feed_buff: &Vec<NeoUVec3> = &feedback_buffer.data;

    // let mut end_for: usize = 0;
    // if(var_dat.feedback_idx<FEEDBACK_BUFFER_SIZE as u32){
    //     end_for = var_dat.feedback_idx as usize;
    // }else{
    //     end_for = FEEDBACK_BUFFER_SIZE;
    // }
    

    // for i in 0..end_for {

    //     print!("{},{},{} -",feed_buff[i as usize].x,feed_buff[i as usize].y,feed_buff[i as usize].z);

    //     continue;
    // }
    // println!("new line");
    // var_dat.feedback_idx = 0;


    //eliminar los NeoUVec3 repetidos para enviarselos al voxel_engine
    //? puede que eliminar los repetidos sea inecesario, pero ayuda al momento de generar los chunks
    let var_dat: &mut VarData = &mut variable_data.data;
    let feed_buff: &Vec<NeoUVec3> = &feedback_buffer.data;

    if var_dat.feedback_idx != 0 {
        let mut feedback_packet: Vec<NeoUVec3> = Vec::new();
        feedback_packet.push(feed_buff[0].clone());
        'outer: for i in 1..min(FEEDBACK_BUFFER_SIZE,(var_dat.feedback_idx) as usize) {
            for nvec in feedback_packet.iter(){
                if feed_buff[i].x == nvec.x && feed_buff[i].y == nvec.y && feed_buff[i].z == nvec.z {
                    continue 'outer;
                }
            }
            feedback_packet.push(feed_buff[i].clone());
        }


        //aqui enviar al voxel_engine
        ev_cellrequest.send(CellsRequest(feedback_packet));
    }

    //necesaro para que el shader pueda rellenar el buffer desde el inicio
    var_dat.feedback_idx = 0;

}

//agregar commandoIdx al vardata
#[repr(C)]
#[derive(ShaderType,Debug,Pod,Zeroable,Clone, Copy)]
pub struct Comando{
    pub allocar: BMAlloc,
    pub deallocar: u32,
    pub datos: Brickmap,
    pub com : u32
}
impl Default for Comando{
    fn default() -> Self {
        Comando{
            allocar: BMAlloc{bm_idx:0,bm_buffer_idx:0},
            deallocar: 0,
            datos: Brickmap::default(),
            com: 0
        }
    }
}

//com = 1 -> tiene datos
//com = 0 -> no tiene datos de brickmap

#[repr(C)]
#[derive(ShaderType,Debug,Pod,Zeroable,Clone, Copy)]
pub struct BMAlloc {
    pub bm_idx: u32,
    pub bm_buffer_idx: u32
}

#[derive(Resource)]
pub struct ObjectPool{
    pub data: Vec<BMAlloc>,
    pub indice: u32
}

#[derive(Resource)]
pub struct CommandBuffer{
    pub comandos: Vec<Comando>
}

fn cell_receiver_allocator(
    mut ev_cellresponse: EventReader<CellsResponse>,
    compute_worker: ResMut<AppComputeWorker::<VoxelRenderWorker>>,
    mut command_buffer: ResMut<CommandBuffer>,
    mut object_pool: ResMut<ObjectPool>
) {
    if !compute_worker.ready() {
        return
    }
    //DO SOMETHING


    let mut comandos: Vec<Comando>= Vec::new();
    for ev in ev_cellresponse.read() {
        for bm in &ev.0{

            //TODO: AQUI CREAR COMANDO SI EL BRICKMAP ESTA VACIO
            //USAR OPTION

            match bm.0 {
                None => {
                    
                    //este comando hara que se aloquen solo en el brickmap grig un puntero que dira vacio
                    comandos.push(Comando { allocar: BMAlloc { bm_idx: bm.1, bm_buffer_idx: 0 }, deallocar: bm.1, datos: Brickmap::cpu_to_gpu(BrickMap::default()), com: 0 });
                    continue;

                },
                Some(brick) => {

                    //aqui allocar en el object_pool y enviar comando para que el shader lo aloque en la grid
                let indice = object_pool.indice;
            
                //si object_pool.data es mayor o igual que OBJECT_POOL_MAX_SIZE significa que hay que reemplazar un brickmap en la posicion indice
                //de ser menor, solo se agrega al vector
                if object_pool.data.len()+1 == OBJECT_POOL_MAX_SIZE {
                    //reemplazar
                    //añade el BMAlloc al final del vector
                    object_pool.data.push(BMAlloc { bm_idx: bm.1, bm_buffer_idx: indice });
                    //quita el BMAlloc donde indica el indice y lo reemplaza por el ultimo, que en este caso es el que se añadio anteriormente
                    let removido = object_pool.data.swap_remove(indice as usize);

                    comandos.push(Comando { allocar: BMAlloc { bm_idx: bm.1, bm_buffer_idx: indice }, deallocar: removido.bm_idx, datos: Brickmap::cpu_to_gpu(brick),com: 1 });
                

                }else {
                    //solo agregar
                    object_pool.data.push(BMAlloc { bm_idx: bm.1, bm_buffer_idx: indice });

                    //si allocar y deallocar son iguales, no habra deallocacion en la gpu
                    comandos.push(Comando { allocar: BMAlloc { bm_idx: bm.1, bm_buffer_idx: indice }, deallocar: bm.1, datos: Brickmap::cpu_to_gpu(brick),com: 1 });
                }


                //aumentar el indice para que la proxima allocacion sea en una posicion diferente
                if object_pool.indice < (OBJECT_POOL_MAX_SIZE -1) as u32 {
                    object_pool.indice += 1;
                }else {
                    object_pool.indice = 0;
                }

                    }
            }


            
            
        }
    }

    //reemplazar command_buffer con el nuevo vector comandos
    command_buffer.comandos = comandos;




}


//? separar buffer_to_resource() y resource_to_buffer() por cada buffer ? o mantenerlos unidos?
fn buffer_to_resource(
    mut variable_data: ResMut<VariableData>,
    mut feedback_res: ResMut<Feedback>,
    compute_worker: ResMut<AppComputeWorker::<VoxelRenderWorker>>
) {

    if !compute_worker.ready() {
        return
    }
    
    
    let var_dat: VarData = compute_worker.try_read("variable_data").unwrap();
    variable_data.data = var_dat;

    let feedback_buffer_data : Vec<NeoUVec3> = compute_worker.try_read_vec("feedback").unwrap();
    feedback_res.data = feedback_buffer_data;

    
}

fn resource_to_buffer(
    mut variable_data: ResMut<VariableData>,
    mut command_buffer: ResMut<CommandBuffer>,
    mut compute_worker: ResMut<AppComputeWorker::<VoxelRenderWorker>>
) {

    if !compute_worker.ready() {
        return
    }

    //vaiable_data
    //...

    
    
    //Command_buffer
    let mut comandos = command_buffer.comandos.clone();
    let com_len = comandos.len();
    if com_len >= 255{
        println!("Demasiados comandos {:?}",com_len);
    }
    //let mut com_pad = vec![Comando::default();256 - command_buffer.comandos.len()];
    //comandos.append(&mut com_pad);
    //el buffer de los comandos deve ser el doble de grande que el feedvack devido a que pueden quedarse 
    let mut aux = [Comando::default();FEEDBACK_BUFFER_SIZE*2];
    for i in 0..com_len{
        aux[i]= if let Some(x) = comandos.pop() {x} else {continue};
    }
    variable_data.data.command_buffer_size = com_len as u32;
    



    //CUALQUIER OTRA COSA




    //ESCRITURA
    compute_worker.write("variable_data", &variable_data.data);
    compute_worker.write("commands_pool",&aux);

}

//la variable time es inecesaria por ahora
fn vardata_actualisation(
    time: Res<Time>,
    mut variable_data: ResMut<VariableData>,
    compute_worker: ResMut<AppComputeWorker::<VoxelRenderWorker>>,
) {
    if !compute_worker.ready() {
        return
    }

    variable_data.data.time = time.elapsed().as_secs() as u32;
}

fn deteccion_movimiento(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut evr_motion: EventReader<MouseMotion>,
    mut exit: EventWriter<AppExit>,
    compute_worker: ResMut<AppComputeWorker::<VoxelRenderWorker>>,
    mut variable_data: ResMut<VariableData>,
    mut camera_t: Query<&mut Transform, (With<MovementCamera>)>,
) {
    if !compute_worker.ready() {
        return
    }

    let Ok(mut camera_t) = camera_t.get_single_mut() else {
        return;
    };

    let var_dat: &mut VarData = &mut variable_data.data;
    

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
    let movimineto = direccion.normalize_or_zero() * time.delta_seconds() * 40.;//multiplicar por un escalar para hacer el movimiento mas rapido/lento
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
        camera_t.rotate_y(-ev.delta.x * 0.003);
        camera_t.rotate_local_x(-ev.delta.y * 0.003);
        
    }
    let forward = camera_t.forward().as_vec3(); 


    //guardar datos en variable
    var_dat.direction = NeoVec3::nuevo(forward.x,forward.y,forward.z);
    var_dat.source = NeoVec3::nuevo(camera_t.translation.x,camera_t.translation.y,camera_t.translation.z);
    var_dat.camera_mat = NeoMat4::from_mat4(camera_t.compute_matrix());
    
    ////!quitar tod lo que tenga que ver con matrix
    //println!("{:?} {:?}",NeoMat4::from_mat4(camera_t.compute_matrix()),camera_t.compute_matrix());

    //println!("source: {:?} + direcction: {:?}",var_dat.source,var_dat.direction);
    let mut cuat = camera_t.rotation.to_euler(EulerRot::XYZ);
    cuat.0 = (cuat.0*180.)/PI;
    cuat.1 = (cuat.1*180.)/PI;
    cuat.2 = (cuat.2*180.)/PI;

    //println!("{:?}",cuat);

    //println!("translation: {:?}, forward: {:?}",camera_t.translation,camera_t.forward().as_vec3());
    
    //escribir variables en bvuffer
    //compute_worker.write("variable_data", &var_dat);



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
            transform: Transform::from_xyz(126., 126., 126.),
                //.looking_at(-Vec3::Z, Vec3::Y),//esto hace que la camara mire hacia una ezquina!?
            
            camera_3d: Camera3d {
                ..default()
            },
            ..default()
        },
        MovementCamera
    ));


}

fn cursor_grab(
    mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut primary_window = q_windows.single_mut();

    primary_window.cursor.grab_mode = CursorGrabMode::Locked;

    primary_window.cursor.visible = false;
}

#[derive(TypePath)]
struct VoxelRenderShader;
impl ComputeShader for VoxelRenderShader {
    fn shader() -> ShaderRef {
        COMPUTE_SHADER_PATH.into()
    }
}

#[derive(TypePath)]
struct VoxelAllocatorShader;
impl ComputeShader for VoxelAllocatorShader {
    fn shader() -> ShaderRef {
        ALLOCATOR_SHADER_PATH.into()
    }
}

//ALLOCATOR_SHADER_PATH

//cambiar dependiendo de la vram de la gpu y el ratio brickmap-=-colorData
const BD_SIZE: usize = OBJECT_POOL_MAX_SIZE;
//antes era 1000

#[derive(Resource)]
struct VoxelRenderWorker;
impl ComputeWorker for VoxelRenderWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        // let init_data: InitData= InitData{
        //     imagen_height: SIZE.0/64,
        //     imagen_width: SIZE.1/64,
        //     feedback_buffer_size: FEEDBACK_BUFFER_SIZE as u32,
        //     world_size: NeoUVec4 { x: WORLD_SIZE.0, y: WORLD_SIZE.1, z: WORLD_SIZE.2, w:0}
        // };
        let init_data: InitData= InitData{
            imagen_height: SIZE.0/64,
            imagen_width: SIZE.1/64,
            feedback_buffer_size: FEEDBACK_BUFFER_SIZE as u32,
            world_size_x: WORLD_SIZE.0,
            world_size_y: WORLD_SIZE.1,
            world_size_z: WORLD_SIZE.2
        };

        let mut var_data: VarData= VarData{
            source: NeoVec3::nuevo(50.,50.,10.),//Vec3 { x: 0., y: 0., z: 0. },
            direction: NeoVec3::nuevo(0.,0.,-1.),//Vec3 { x: 0., y: 0., z: -1. },
            fov: 50.,
            camera_mat: NeoMat4::IDENTITY(),
            time: 0,
            feedback_idx: 0,
            command_buffer_size: 0
        };
        var_data.source.x = 126.;
        var_data.source.y = 126.;
        var_data.source.z = 126.; 

        let blank_image = &[0 as u32;(SIZE.0 as usize)*(SIZE.1 as usize)];


        const ws :u32 = WORLD_SIZE.0*WORLD_SIZE.1*WORLD_SIZE.2;

        //byte 1 2 y 3 = Rgb LOD, byte 4 = flags
        //flags 00000000 = empty
        const def_brickgrid_cell: u32= 0x355f9502;//0b00000000_00000000_00000000_00000000

        #[repr(C)]
        #[derive(ShaderType,Debug,AnyBitPattern,Clone, Copy)]
        struct ato {
            at: u32
        }

        //let gri:Vec<u32> = test_brickgrid(WORLD_SIZE, ws);
        // [def_brickgrid_cell;ws as usize]
        //let test_grid = &gri[..16777216];


        let camera_transform: Mat4 = Mat4::IDENTITY;

        let worker = AppComputeWorkerBuilder::new(world)
        .add_uniform("uniform_data",&init_data)
        .add_staging("variable_data", &var_data)
        .add_staging("imagen", &blank_image)
        .add_rw_storage("brickgrid", &[def_brickgrid_cell;ws as usize])
        .add_rw_storage("brickmap_data", &[Brickmap::default();BD_SIZE])
        .add_staging("feedback", &[NeoUVec3::default();FEEDBACK_BUFFER_SIZE])
        .add_staging("commands_pool",&[Comando::default();FEEDBACK_BUFFER_SIZE *2])
        .add_pass::<VoxelAllocatorShader>([FEEDBACK_BUFFER_SIZE as u32 *2,1,1], &["variable_data","commands_pool","brickgrid","brickmap_data"])
        .add_pass::<VoxelRenderShader>([512,512,1],&["uniform_data","variable_data","imagen","brickgrid","brickmap_data","feedback"])
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


