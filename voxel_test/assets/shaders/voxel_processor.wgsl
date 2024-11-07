struct brickmap {
    datos: u32,
}

struct InitData {
    imagen_height: u32,
    imagen_width: u32
}

struct NeoVec3{
    x: f32,
    y: f32,
    z: f32
}

struct NeoUVec3{
    x: u32,
    y: u32,
    z: u32
}

struct VarData {
    cam_src : NeoVec3,
    cam_dir: NeoVec3,
    fov: f32,
    used_buff: atomic<u32>,//usar como contador, a que espacio de memoria se puede escribir en el feedback_buffer y/o hasta cual esta ocupado
    time: u32
}

struct Ray {
    source: vec3<f32>,
    direction: vec3<f32>
}

struct Camera {
    eye: vec3<f32>,
    targeting: vec3<f32>,
    fov: f32
}

struct Test {
    atomi: atomic<u32>
}

//este se usara como recipiente para todos los datos unicos necesarios de solo lectura
//obtenibles al iniciar la aplicacion (no se pueden cambiar despues)
@group(0) @binding(0)
var<uniform> init_data: InitData;

@group(0) @binding(1)
var<storage, read_write> var_dat: VarData;

//usar color en Hexadecimal
@group(0) @binding(2)
var<storage, read_write> imagen: array<u32>;

@group(0) @binding(3)
var<storage, read_write> brickgrid: array<u32>;// grid de 256x256x256 POR AHORA

@group(0) @binding(4)
var<storage, read_write> brickmap_data: array<brickmap>;//brickmaps cargador

@group(0) @binding(5)
var<storage, read_write> feedback : array<NeoUVec3>; // feedback de solo escritura en gpu



@compute @workgroup_size(1,1,1)//64
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let pixel: vec2<u32> = vec2<u32>(invocation_id.x,invocation_id.y);

    //esto es solo para mantener vivas las conecciones a los storage
    //tienen que usarse al menos una vez en el shader
    if (invocation_id.x==0){
        let dum: u32 = init_data.imagen_height;
        let dim: f32 = var_dat.cam_src.x;
        let dom: u32 = imagen[0];
        let dem: u32 = brickgrid[0];
        //let dem: u32 = atomicLoad(&brickgrid[0]);
        let dam: u32 = brickmap_data[0].datos;
        let uve: NeoUVec3 = feedback[0];
        //let ato: u32 = atomicLoad(&atomico.atomi);
    }

    /*


    asign_color(pixel,0x00ffffffu);
    //imagen[invocation_id.x+(512*invocation_id.y)] = 0x00ffffffu;
    if(var_dat.time%2< 1 && invocation_id.x<60){
        asign_color(pixel,0xffffffffu);
        //imagen[invocation_id.x+(512*invocation_id.y)] = 0xffffffffu;
    }
    if(var_dat.time%2== 1 && invocation_id.x>400){
        asign_color(pixel,0xffffffffu);
        //imagen[invocation_id.x+(512*invocation_id.y)] = 0xffffff00u;
    }
    if(invocation_id.x>200 && invocation_id.x<300){
        //asign_color(pixel,atomicLoad(&atomico.atomi));
        //imagen[invocation_id.x+(512*invocation_id.y)] = 0xffffff00u;
    }

    */

    ///inisialisar datos para el raymarching
    //crear rayo calculando el viewmatrix ¿normalisar direccion?

    let camsrc: vec3<f32> = vec3<f32>(var_dat.cam_src.x,var_dat.cam_src.y,var_dat.cam_src.z);
    let camdir: vec3<f32> = vec3<f32>(var_dat.cam_dir.x,var_dat.cam_dir.y,var_dat.cam_dir.z);

    let camara: Camera = Camera(camsrc,camdir,var_dat.fov);

    var rayo :Ray = crear_rayo(invocation_id.x,invocation_id.y,512u,512u,camara);

    let hit = raymarching(rayo,pixel);


    if (hit) {
        //asign_color(pixel,0x00ffffffu);
    } else {
        asign_color(pixel,0x0000ffffu);
    }


    //algoritmo de raymarching a travez del brickgrid
    //loop? for?


    //if si el brickmap esta cargado o no
    //true= 


    //end loop/for


}

fn neo_crear_rayo(x: u32, y: u32, image_width: u32, image_height: u32, camera: Camera) -> Ray {
    //no usar aspect ratio

    let viewport_height: f32 = 2.;
    let viewport_width: f32 = 2.;

    let focal_length: f32 = 1.;

    let camera_center: vec3<f32> = camera.eye;

    //calcular vector a travez de las ezquinas horizontales y verticales
    // u = width     v=heigth
    let viewport_u: vec3<f32>= vec3<f32>(viewport_width,0.,0.);
    let viewport_v: vec3<f32>= vec3<f32>(0.,viewport_height,0.);

    // Calculate the horizontal and vertical delta vectors from pixel to pixel.
    let pixel_delta_u: vec3<f32>= viewport_u / f32(image_width);
    let pixel_delta_v: vec3<f32>= viewport_v / f32(image_height);

    // Calculate the location of the upper left pixel.
    let viewport_upper_left: vec3<f32>= camera_center - vec3<f32>(0., 0., focal_length) - viewport_u/2 - viewport_v/2;







}

fn crear_rayo(x: u32, y: u32, width: u32, height: u32, camera: Camera) -> Ray{
    //X y Y son el numero de rayos por axis
    //height y width son el numero de pixeles por axis?

    let aspect: f32 = f32(width) / f32(height); //aspect ratio
    let theta: f32 = radians(camera.fov);
    let half_height = tan(theta/2.0);
    let half_width = aspect * half_height;

    let w: vec3<f32> = normalize(camera.eye-normalize(camera.targeting));
    let u: vec3<f32> = normalize(cross(vec3<f32>(0.,1.,0.),w));
    let v: vec3<f32> = cross(w,u);

    let origin: vec3<f32> = camera.eye;
    let lower_left_corner: vec3<f32> = origin - (u*half_width) - (v*half_height) - w;
    let horizontal: vec3<f32> = u * 2. *half_width;
    let vertical: vec3<f32> = v * 2. * half_height;


    let xu: f32 = f32(x)/f32(width);
    let yv: f32 = f32(y)/f32(height);
    let dir: vec3<f32> = normalize(lower_left_corner + (horizontal*xu) + (vertical*yv) - origin);


    return Ray(origin, dir);
}


//devolvera falso si no golpea nada
fn raymarching(ray: Ray, pixel: vec2<u32>) -> bool {
    let MAX_RAY_STEPS: u32 = 50u;
    var rayo = ray;

    //borrar los que no se utilizen
    var o_hit_axis: vec3<bool> = vec3<bool>(false,false,false); //axis del hit
    //var o_hit_dist: vec3<f32> = vec3<f32>(0.,0.,0.); // distancia hasta donde dio un hit
    var o_hit_vox: vec3<i32> = vec3<i32>(0,0,0); //voxel al cual se golpeo !!!!!!
    var o_hit_pos: vec3<f32> = vec3<f32>(0.,0.,0.); // posicion global del hit?
    var o_hit_uvw: vec3<f32> = vec3<f32>(0.,0.,0.); // posicion del hit, en el voxel
    var o_hit_nor: vec3<f32> = vec3<f32>(0.,0.,0.); //normal de la cara del voxel golpeado

    rayo.direction = normalize(rayo.direction)+0.0001;
    
    var ray_signf: vec3<f32> = sign(rayo.direction);
    var ray_sign: vec3<i32> = vec3<i32>(i32(ray_signf.x),i32(ray_signf.y),i32(ray_signf.z));
    var ray_step: vec3<f32> = 1./rayo.direction; //si no funciona, hacerlo por cada valor;
    
    var ray_origin_grid: vec3<f32> = floor(rayo.source);
    var voxel_coords: vec3<i32> = vec3<i32>(i32(ray_origin_grid.x),i32(ray_origin_grid.y),i32(ray_origin_grid.z));

    var side_distance: vec3<f32> = ray_origin_grid - rayo.source;
    side_distance = side_distance +0.5;
    side_distance = side_distance + (ray_signf * 0.5);
    side_distance = side_distance * ray_step;

    var mask: vec3<bool> = vec3<bool>(false,false,false);


    for (var i = 0u; i < MAX_RAY_STEPS; i++) {

        if(voxel_coords.x>255 || voxel_coords.y>255 || voxel_coords.z>255 || voxel_coords.x<0 || voxel_coords.y<0 || voxel_coords.z<0){
            asign_color(pixel,0x00ff00ffu);
            return true;
        }

        if(is_voxel_filled(voxel_coords) != 0u){
            
            /*
            if(i==0){
                asign_color(pixel,0x00ff00ffu);
                return false;
            }
            */
            //o_hit_axis = mask; //redundante

            //determina el hit final en espacio global ¿inecesario?
            o_hit_pos = side_distance - rayo.source;
            o_hit_pos = o_hit_pos + 0.5;
            o_hit_pos = o_hit_pos - (ray_signf * 0.5);
            o_hit_pos = o_hit_pos * ray_step;
            //o_hit_pos = ray_origin + ray_direction * o_hit_dist; // comentado en el ejemplo

            // voxel en hit
            o_hit_vox = voxel_coords;


            // distancia hasta el hit
            //o_hit_dist = max(o_hit_pos.x, max(o_hit_pos.y, o_hit_pos.z)); // inecesaria por ahora


            //la normal de la cara del hit// si el hit sucede en el primer step entonses la normal es inutil
            o_hit_nor = vec3<f32>(f32(mask.x),f32(mask.y),f32(mask.z)) * -ray_signf;


            //la posicion del hit en el voxel
            //o_hit_uvw = o_hit_pos - o_hit_vox; //cambiar tipos para que concuerden


            //aplicar color (por ahora es plano pero luego aplicara la normal del voxel)
            asign_color(pixel,is_voxel_filled(voxel_coords));


            return true;
            //para implemenar la grid de orden mayor, se ejecuta raymarching() de forma recursiva;;
        }

        //mask = lessThanEqual(side_distance.xyz, min(side_distance.yzx, side_distance.zxy));

        var mini: vec3<f32> = min(side_distance.yzx, side_distance.zxy);

        mask = vec3<bool>(side_distance.x<=mini.x, side_distance.y<=mini.y, side_distance.z<=mini.z);

        side_distance = side_distance + (vec3<f32>(f32(mask.x),f32(mask.y),f32(mask.z)) * ray_step);

        voxel_coords = voxel_coords + (vec3<i32>(i32(mask.x),i32(mask.y),i32(mask.z)) * ray_sign);


    }








    return false;
}

//por ahora lee el color de brickgrid pero es solo para testing
fn is_voxel_filled(coordenadas: vec3<i32>) -> u32 {
    //revisar brickgrid si es que tiene color o no; .....
    let coords: vec3<u32> = vec3<u32>(u32(coordenadas.x),u32(coordenadas.y),u32(coordenadas.z));

    // el tamaño del grid es POR AHORA 256x256x256
    var color: u32= 0x00000000u;
    //let color: u32 = brickgrid[u32(coordenadas.x) + (255*(u32(coordenadas.y)+ (255 * u32(coordenadas.z))))];

    //SOLO POR TEST
    if(coordenadas.z==101 && 0<coordenadas.y && coordenadas.y<300 && 0<coordenadas.x && coordenadas.x<300) {
        //grid[(x + world.1 * (y +(world.2 * z))) as usize] = 0xff0000ff;
        color = 0xff0000ffu;
    }
    if(coordenadas.z==100 && 10<coordenadas.y && coordenadas.y<50 && 10<coordenadas.x && coordenadas.x<120) {
        //grid[(x + world.1 * (y +(world.2 * z))) as usize] = 0xff0000ff;
        color = 0xff00ffffu;
    }


    return color;


}


fn asign_color(pixel: vec2<u32>, color: u32) {
    //pixel.x = pixel.x.mod(512.)
    imagen[pixel.x+(512*pixel.y)]= color;

}

// para comparar un u32 y un f32 se hace f32(variable) que es lo mismo que "variable as f32"