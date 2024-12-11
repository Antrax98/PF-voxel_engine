struct brickmap {
    datos: array<u32,16>,
}

struct InitData {
    imagen_height: u32,
    imagen_width: u32,
    feedback_buffer_size : u32
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

struct NeoVec4{
    x: f32,
    y: f32,
    z: f32,
    w: f32
}

struct NeoMat4{
    x_axis: NeoVec4,
    y_axis: NeoVec4,
    z_axis: NeoVec4,
    w_axis: NeoVec4,
}

struct VarData {
    cam_src : NeoVec3,
    cam_dir: NeoVec3,
    fov: f32,
    cam_transform: NeoMat4,
    time: u32,
    feedback_idx: atomic<u32>,//usar como contador, a que espacio de memoria se puede escribir en el feedback_buffer y/o hasta cual esta ocupado
    command_buffer_size: u32,
}

struct Ray {
    source: vec3<f32>,
    direction: vec3<f32>
}

struct Camera {
    eye: vec3<f32>,
    targeting: vec3<f32>,
    fov: f32,
    transform: NeoMat4//mat4x4<f32>//eliminar??
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
var<storage, read_write> brickgrid: array<u32>;// grid de 256x256x256 Cells POR AHORA

@group(0) @binding(4)
var<storage, read_write> brickmap_data: array<brickmap>;//brickmaps alocados

@group(0) @binding(5)
var<storage, read_write> feedback : array<NeoUVec3>; // feedback de solo escritura en gpu

const pi = radians(180.0);

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
        let dam: u32 = brickmap_data[0].datos[0];
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

    let camara: Camera = Camera(camsrc,camdir,var_dat.fov,var_dat.cam_transform);

    var rayo :Ray = crear_rayo(invocation_id.x,invocation_id.y,512u,512u,camara);

    let hit = neo_raymarching(rayo,pixel);


    if (hit) {
        //asign_color(pixel,0x00ffffffu);
    } else {
        asign_color(pixel,0x674ea7ffu);//MORADO
    }


    //algoritmo de raymarching a travez del brickgrid
    //loop? for?


    //if si el brickmap esta cargado o no
    //true= 


    //end loop/for


}

// fn mult_matrix_vector(matrix: mat4x4<f32>, vec3<f32>)

fn neo_crear_rayo(x: u32, y: u32, image_width: u32, image_height: u32, camera: Camera) -> Ray {
    //no usar aspect ratio

    ///let aspect_ratio f32= f32(image_width)/f32(image_height);






    /*
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
    */

    //0.5 is a shift to align te pixel to te viewport
    //NDC space results will be in [0,1]
    let p_ndc_x: f32 = (f32(x) + 0.5) / f32(image_width);
    let p_ndc_y: f32 = (f32(y) + 0.5) / f32(image_height);

    //Screen space results will be [-1,1]
    let p_screen_x = 1 - 2 * p_ndc_x -1;
    let p_screen_y = 1 - 2 * p_ndc_y -1;

    //se asume que width es mayor que height
    let aspect_ratio: f32 = f32(image_width)/f32(image_height);

    //camera space with fov into account
    let pixel_camera_x: f32 = (2* (p_screen_x -1))* aspect_ratio * tan(camera.fov/2);
    let pixel_camera_y: f32 = 1-(2*p_screen_y) * tan(camera.fov/2);

    // Pixel Camera Space vec
    let pcs: vec3<f32> = vec3<f32>(pixel_camera_x,pixel_camera_y,-1.);

    
    //let ctw/camera_to_world: mat4x4<f32> = camera.transform; //usar si NeoMat4 no funciona
    let ctw: NeoMat4 = camera.transform;
    
    /*
    let rayPWorld: vec3<f32> = vec3<f32>(
        ctw.x_axis[0]*pcs.x + ctw.x_axis[1]*pcs.y + ctw.x_axis[2]*pcs.z,
        ctw.y_axis[0]*pcs.x + ctw.y_axis[1]*pcs.y + ctw.y_axis[2]*pcs.z,
        ctw.z_axis[0]*pcs.x + ctw.z_axis[1]*pcs.y + ctw.z_axis[2]*pcs.z
    );
    */

    let rayPWorld: vec3<f32> = vec3<f32>(
        ctw.x_axis[0]*pcs.x + ctw.y_axis[0]*pcs.y + ctw.z_axis[0]*pcs.z + ctw.w_axis[0],
        ctw.x_axis[1]*pcs.x + ctw.y_axis[1]*pcs.y + ctw.z_axis[1]*pcs.z + ctw.w_axis[1],
        ctw.x_axis[2]*pcs.x + ctw.y_axis[2]*pcs.y + ctw.z_axis[2]*pcs.z + ctw.w_axis[2]
    );


    //PRUEBA: multiplicar forward(targeting) con la direccion del PCS y entregar eso como resultado
    //de no funcionar, hacerlo con el mat4
    //let rayPWorld: vec3<f32> = camera.targeting*pcs;
    
    let ray_dir: vec3<f32> = rayPWorld-camera.eye;
    
    //let ray_dir: vec3<f32> = vec3<f32>(pcs.x,pcs.y,-1) - camera.targeting;//probar quitando camera.targeting para simular que existe en el origen


    let ray: Ray = Ray(camera.eye,ray_dir);


}

fn new_neo_crear_rayo(x: u32, y: u32, image_width: u32, image_height: u32, camera: Camera) -> Ray {
    
    
    //0.5 is a shift to align te pixel to te viewport
    //NDC space results will be in [0,1]
    let p_ndc_x: f32 = (f32(x) + 0.5) / f32(image_width);
    let p_ndc_y: f32 = (f32(y) + 0.5) / f32(image_height);

    //Screen space results will be [-1,1]
    let p_screen_x = 1 - 2 * p_ndc_x -1;
    let p_screen_y = 1 - 2 * p_ndc_y -1;

    //se asume que width es mayor que height
    let aspect_ratio: f32 = f32(image_width)/f32(image_height);

    //camera space with fov into account
    let pixel_camera_x: f32 = (2* (p_screen_x -1))* aspect_ratio * tan(camera.fov/2);
    let pixel_camera_y: f32 = 1-(2*p_screen_y) * tan(camera.fov/2);

    // Pixel Camera Space vec
    let pcs: vec3<f32> = vec3<f32>(pixel_camera_x,pixel_camera_y,-1.);


    //cross mult forward_dir(camera.targeting) x UpDir (y positivo)
    let upDirection :vec3<f32> = vec3<f32>(0.,1.,0.);
    let rightDirection :vec3<f32> = cross(camera.targeting,upDirection); 










}

//ahora parece funcionar
fn crear_rayo(x: u32, y: u32, width: u32, height: u32, camera: Camera) -> Ray{
    //X y Y son el numero de rayos por axis
    //height y width son el numero de pixeles por axis?

    let aspect: f32 = f32(width) / f32(height); //aspect ratio
    let theta: f32 = radians(camera.fov);
    let half_height = tan(theta/2.0);
    let half_width = aspect * half_height;

    //let w: vec3<f32> = normalize(camera.eye-normalize(camera.targeting));
    var w: vec3<f32> = normalize(camera.targeting);//vector hacia delante de la camara
    //w.z = w.z *-1.;
    //w.x = w.x *-1.;
    let u: vec3<f32> = normalize(cross(vec3<f32>(0.,1.,0.),w));//vector horizontal a la camara
    let v: vec3<f32> = cross(w,u);// vector vertical? a la camara

    let origin: vec3<f32> = camera.eye;
    let lower_left_corner: vec3<f32> = origin - (u*half_width) - (v*half_height) - w;
    let horizontal: vec3<f32> = u * 2. * half_width;
    let vertical: vec3<f32> = v * 2. * half_height;


    let xu: f32 = f32(x)/f32(width);
    let yv: f32 = f32(y)/f32(height);
    //cambiar la direccion a negativa parece arreglar el problema de los axis
    var dir: vec3<f32> = -normalize(lower_left_corner + (horizontal*xu) + (vertical*yv) - origin);


    return Ray(origin, dir);
}


//a primera vista, el cambio de algoritmo no dio un cambio significativo al rendimiento
fn nc_neo_raymarching(ray: Ray, pixel: vec2<u32>) -> bool {

    var rayo: Ray = ray;

    let MAX_RAY_STEPS: u32 = 500u;

    //TODO: obtener coordenadas en caso de que el rayo se origine fuera del mundo voxel
    //ejemplo: de tener un mundo cubico de 250+, si el rayo se origina desde una coordenada negativa,
    //este estaria fuera del mundo y seria necesario un nuevo proceso para obtener a cual voxel apunta primero.

    var ray_origin_grid: vec3<f32> = floor(ray.source); // starting voxel coordinates
    

    //esto se encarga de, en caso que xyz no sean igual a zero
    if(rayo.direction.x == 0.0){
        rayo.direction.x += 0.0001;
    }
    if(rayo.direction.y == 0.0){
        rayo.direction.y += 0.0001;
    }
    if(rayo.direction.z == 0.0){
        rayo.direction.z += 0.0001;
    }
    rayo.direction = normalize(rayo.direction);


    //aqui se obtiene el signo de cada valor de la direccion, de modo que se sepa si se debe aumentar o disminuir
    //en el axis que se este calculando en el momento
    //* el vector ray_sign y ray_signf son lo mismo que stepX,stepY,stepZ en un solo vector (habra que separarlos???)
    var ray_signf: vec3<f32> = sign(rayo.direction);
    var ray_sign: vec3<i32> = vec3<i32>(i32(ray_signf.x),i32(ray_signf.y),i32(ray_signf.z));
    //var ray_step: vec3<f32> = 1./rayo.direction; //si no funciona, hacerlo por cada valor;

    



    //t_delta xyz deverian multiplicarse por el tamaño del voxel, pero como por ahora el voxel es de tamaño 1 no hay que multiplicarlo //*(por ahora)
    var t_delta: vec3<f32> = vec3<f32>(
        length(rayo.direction * (1/rayo.direction.x)),
        length(rayo.direction * (1/rayo.direction.y)),
        length(rayo.direction * (1/rayo.direction.z))
    );

    let offset: vec3<f32> = ray_signf - (ray.source - ray_origin_grid);//hecho por mi //? puede estar roto


    //este representa al t_max solo al inicio, ya que sera modificado dentro del loop
    var t_max: vec3<f32> = t_delta * offset;

    //coordenadas del voxel en las que se comiensa
    var voxel_coords: vec3<i32> = vec3<i32>(i32(ray_origin_grid.x),i32(ray_origin_grid.y),i32(ray_origin_grid.z));

    //*DESDE AQUI ES NUEVO

    var voxel_incr: vec3<bool> = vec3<bool>(false,false,false);
    
    for(var i=0u; i< MAX_RAY_STEPS;i++){

        //!necesario crear una solucion similar para cada nivel (parecido a como se hara con is_voxel_filled() )
        if(voxel_coords.x>255 || voxel_coords.y>255 || voxel_coords.z>255 || voxel_coords.x<0 || voxel_coords.y<0 || voxel_coords.z<0){
            asign_color(pixel,0x00ff00ffu);//VERDE
            return true;
        }

        //! SOLO POR AHORA
        if(work_brickcell(voxel_coords)){
            return true;
        }

        //! cambiar como funciona is_voxel_filled() dependieondo del nivel en el que este (crear una funcion por cada nivel???)
        //brickmap -> revisara dentro del brickmap si el voxel el true o false (revisa el mismicimo brickmap)
        //brickcell -> revisara si existe un brickmap o si es una cell vacia (revisa dentro de las alocaciones)
        //celda superior 1 -> revisara si existe algun brickcell o si esta vacia (esta es solo un true/false)
        //TODO: de existir algo, se deve adentrarse e iniciar un nuevo 3d dda
        //? explorar como interpretar la escala del mundo (asi como multiplicar o dividir la pocicion de la camara y los rayos por cada nivel)
        if(is_voxel_filled(voxel_coords) != 0u){
            //! DEJAR DE UTILIZAR is_voxel_filled() COMO OBTENEDOR DE COLORES Y CREAR UNA FUNCION PARA CADA CASO NECESARIO
            asign_color(pixel,is_voxel_filled(voxel_coords));
            return true;
        }

        voxel_incr.x = (t_max.x<=t_max.y) && (t_max.x<=t_max.z);
        voxel_incr.y = (t_max.y<=t_max.x) && (t_max.y<=t_max.z);
        voxel_incr.z = (t_max.z<=t_max.x) && (t_max.z<=t_max.y);

        t_max.x += f32(voxel_incr.x) * t_delta.x;
        t_max.y += f32(voxel_incr.y) * t_delta.y;
        t_max.z += f32(voxel_incr.z) * t_delta.z;

        voxel_coords.x += i32(voxel_incr.x) * ray_sign.x;
        voxel_coords.y += i32(voxel_incr.y) * ray_sign.y;
        voxel_coords.z += i32(voxel_incr.z) * ray_sign.z;

    }
    return false;
}

//* EL QUE SE USA ACTUALMENTE
//faltan algunos calculos a la hora de hacer el hit
fn neo_raymarching(ray: Ray, pixel: vec2<u32>) -> bool {

    var rayo : Ray = ray;

    let MAX_RAY_STEPS: u32 = 500u;

    //TODO: obtener coordenadas en caso de que el rayo se origine fuera del mundo voxel
    //ejemplo: de tener un mundo cubico de 250+, si el rayo se origina desde una coordenada negativa,
    //este estaria fuera del mundo y seria necesario un nuevo proceso para obtener a cual voxel apunta primero.

    var ray_origin_grid: vec3<f32> = floor(rayo.source); // starting voxel coordinates
    

    //esto se encarga de, en caso que xyz no sean igual a zero
    if(rayo.direction.x == 0.0){
        rayo.direction.x += 0.0001;
    }
    if(rayo.direction.y == 0.0){
        rayo.direction.y += 0.0001;
    }
    if(rayo.direction.z == 0.0){
        rayo.direction.z += 0.0001;
    }
    rayo.direction = normalize(rayo.direction);


    


    //aqui se obtiene el signo de cada valor de la direccion, de modo que se sepa si se debe aumentar o disminuir
    //en el axis que se este calculando en el momento
    //* el vector ray_sign y ray_signf son lo mismo que stepX,stepY,stepZ en un solo vector (habra que separarlos???)
    var ray_signf: vec3<f32> = sign(rayo.direction);
    var ray_sign: vec3<i32> = vec3<i32>(i32(ray_signf.x),i32(ray_signf.y),i32(ray_signf.z));
    //var ray_step: vec3<f32> = 1./rayo.direction; //si no funciona, hacerlo por cada valor;


    //t_delta xyz deverian multiplicarse por el tamaño del voxel, pero como por ahora el voxel es de tamaño 1 no hay que multiplicarlo //*(por ahora)
    var t_delta: vec3<f32> = vec3<f32>(
        length(rayo.direction * (1/rayo.direction.x)),
        length(rayo.direction * (1/rayo.direction.y)),
        length(rayo.direction * (1/rayo.direction.z))
    );

    let offset: vec3<f32> = ray_signf - (ray.source - ray_origin_grid);//hecho por mi //? puede estar roto


    //este representa al t_max solo al inicio, ya que sera modificado dentro del loop
    var t_max: vec3<f32> = t_delta * offset;

    //coordenadas del voxel en las que se comiensa
    var voxel_coords: vec3<i32> = vec3<i32>(i32(ray_origin_grid.x),i32(ray_origin_grid.y),i32(ray_origin_grid.z));


    
    for(var i=0u; i< MAX_RAY_STEPS;i++){

        //!necesario crear una solucion similar para cada nivel (parecido a como se hara con is_voxel_filled() )
        if(voxel_coords.x>255 || voxel_coords.y>255 || voxel_coords.z>255 || voxel_coords.x<0 || voxel_coords.y<0 || voxel_coords.z<0){
            asign_color(pixel,0x00ff00ffu);//VERDE
            return true;
        }


        //! SOLO POR AHORA
        if(work_brickcell(voxel_coords)){
            return true;
        }


        //! cambiar como funciona is_voxel_filled() dependieondo del nivel en el que este (crear una funcion por cada nivel???)
        //brickmap -> revisara dentro del brickmap si el voxel el true o false (revisa el mismicimo brickmap)
        //brickcell -> revisara si existe un brickmap o si es una cell vacia (revisa dentro de las alocaciones)
        //celda superior 1 -> revisara si existe algun brickcell o si esta vacia (esta es solo un true/false)
        //TODO: de existir algo, se deve adentrarse e iniciar un nuevo 3d dda
        //? explorar como interpretar la escala del mundo (asi como multiplicar o dividir la pocicion de la camara y los rayos por cada nivel)
        if(is_voxel_filled(voxel_coords) != 0u){
            //! DEJAR DE UTILIZAR is_voxel_filled() COMO OBTENEDOR DE COLORES Y CREAR UNA FUNCION PARA CADA CASO NECESARIO
            asign_color(pixel,is_voxel_filled(voxel_coords));
            return true;
        }

        if(t_max.x < t_max.y){
            if(t_max.x < t_max.z){
                voxel_coords.x += ray_sign.x;
                t_max.x += t_delta.x;
            }else{
                voxel_coords.z += ray_sign.z;
                t_max.z += t_delta.z;
            }
        }else{
            if(t_max.y < t_max.z){
                voxel_coords.y += ray_sign.y;
                t_max.y += t_delta.y;
            }else{
                voxel_coords.z += ray_sign.z;
                t_max.z += t_delta.z;
            }
        }
    }


    //CALCULAR MAGNITUD = distance(vecZERO, vecToCalculate);//no se si esta bien el orden
    // length(vecToCalculate);
    return false;
}

//funcion que encapsula el trabajo necesario para un Brickcell y navegar el Brickmap
fn work_brickcell(cell_coords: vec3<i32>) -> bool {
    







    //como ingresar coordenadas al feedbackloop

    let buff_idx = atomicAdd(&var_dat.feedback_idx, 1u);

    if(buff_idx<init_data.feedback_buffer_size){
        feedback[buff_idx] = NeoUVec3(u32(cell_coords.x),u32(cell_coords.y),u32(cell_coords.z));
        return true;
    }else{
        return false;
    }


}


//NO SE BORRA SOLO POR REFERENCIA
fn raymarching(ray: Ray, pixel: vec2<u32>) -> bool {
    let MAX_RAY_STEPS: u32 = 500u;
    var rayo = ray;

    //variables donde se guardaran datos importantes
    //borrar los que no se utilizen
    var o_hit_axis: vec3<bool> = vec3<bool>(false,false,false); //axis del hit
    //var o_hit_dist: vec3<f32> = vec3<f32>(0.,0.,0.); // distancia hasta donde dio un hit
    var o_hit_vox: vec3<i32> = vec3<i32>(0,0,0); //voxel al cual se golpeo !!!!!!
    var o_hit_pos: vec3<f32> = vec3<f32>(0.,0.,0.); // posicion global del hit?
    var o_hit_uvw: vec3<f32> = vec3<f32>(0.,0.,0.); // posicion del hit, en el voxel
    var o_hit_nor: vec3<f32> = vec3<f32>(0.,0.,0.); //normal de la cara del voxel golpeado


    
    rayo.direction = normalize(rayo.direction);

    //esto se encarga de, en caso que xyz no sean igual a zero
    if(rayo.direction.x == 0.0){
        rayo.direction.x += 0.0001;
    }
    if(rayo.direction.y == 0.0){
        rayo.direction.y += 0.0001;
    }
    if(rayo.direction.z == 0.0){
        rayo.direction.z += 0.0001;
    }
    
    
    var ray_signf: vec3<f32> = sign(rayo.direction);
    var ray_sign: vec3<i32> = vec3<i32>(i32(ray_signf.x),i32(ray_signf.y),i32(ray_signf.z));
    var ray_step: vec3<f32> = 1./rayo.direction; //si no funciona, hacerlo por cada valor;
    
    var ray_origin_grid: vec3<f32> = floor(rayo.source);
    var voxel_coords: vec3<i32> = vec3<i32>(i32(ray_origin_grid.x),i32(ray_origin_grid.y),i32(ray_origin_grid.z));

    var side_distance: vec3<f32> = ray_origin_grid - rayo.source;
    //side_distance = side_distance +0.5;//?????
    side_distance = side_distance + (ray_signf * 0.5);
    side_distance = side_distance * ray_step;

    var mask: vec3<bool> = vec3<bool>(false,false,false);


    for (var i = 0u; i < MAX_RAY_STEPS; i++) {

        //cambiar numeros mayores por el tamaño maximo del mundo (deve ser una variable)
        if(voxel_coords.x>255 || voxel_coords.y>255 || voxel_coords.z>255 || voxel_coords.x<0 || voxel_coords.y<0 || voxel_coords.z<0){
            asign_color(pixel,0x00ff00ffu);//VERDE
            return true;
        }

        if(is_voxel_filled(voxel_coords) != 0u){
            
            /*
            if(i==0){
                asign_color(pixel,0x00ff00ffu);
                return false;
            }
            */
            //o_hit_axis = mask; //redundante //usar para pruebas despues

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

        //mask = vec3<bool>(side_distance.x<=mini.x, side_distance.y<=mini.y, side_distance.z<=mini.z);
        mask = vec3<bool>(side_distance<=mini);

        side_distance = side_distance + (vec3<f32>(f32(mask.x),f32(mask.y),f32(mask.z)) * ray_step);

        voxel_coords = voxel_coords + (vec3<i32>(i32(mask.x),i32(mask.y),i32(mask.z)) * ray_sign);


    }








    return false;
}

fn modified_raymarching(ray: Ray, pixel: vec2<u32>) -> bool {
    let MAX_RAY_STEPS: u32 = 500u;
    var rayo = ray;

    //variables donde se guardaran datos importantes
    //borrar los que no se utilizen
    var o_hit_axis: vec3<bool> = vec3<bool>(false,false,false); //axis del hit
    //var o_hit_dist: vec3<f32> = vec3<f32>(0.,0.,0.); // distancia hasta donde dio un hit
    var o_hit_vox: vec3<i32> = vec3<i32>(0,0,0); //voxel al cual se golpeo !!!!!!
    var o_hit_pos: vec3<f32> = vec3<f32>(0.,0.,0.); // posicion global del hit?
    var o_hit_uvw: vec3<f32> = vec3<f32>(0.,0.,0.); // posicion del hit, en el voxel
    var o_hit_nor: vec3<f32> = vec3<f32>(0.,0.,0.); //normal de la cara del voxel golpeado


    
    rayo.direction = normalize(rayo.direction);

    //esto se encarga de, en caso que xyz no sean igual a zero
    if(rayo.direction.x == 0.0){
        rayo.direction.x += 0.0001;
    }
    if(rayo.direction.y == 0.0){
        rayo.direction.y += 0.0001;
    }
    if(rayo.direction.z == 0.0){
        rayo.direction.z += 0.0001;
    }
    
    
    var ray_signf: vec3<f32> = sign(rayo.direction);
    var ray_sign: vec3<i32> = vec3<i32>(i32(ray_signf.x),i32(ray_signf.y),i32(ray_signf.z));
    var ray_step: vec3<f32> = 1./rayo.direction; //si no funciona, hacerlo por cada valor;
    
    var ray_origin_grid: vec3<f32> = floor(rayo.source);
    var voxel_coords: vec3<i32> = vec3<i32>(i32(ray_origin_grid.x),i32(ray_origin_grid.y),i32(ray_origin_grid.z));

    var side_distance: vec3<f32> = ray_origin_grid - rayo.source;
    //side_distance = side_distance +0.5;//?????
    //side_distance = side_distance + (ray_signf * 0.5);
    side_distance = side_distance * ray_step;

    var mask: vec3<bool> = vec3<bool>(false,false,false);


    for (var i = 0u; i < MAX_RAY_STEPS; i++) {

        //cambiar numeros mayores por el tamaño maximo del mundo (deve ser una variable)
        if(voxel_coords.x>255 || voxel_coords.y>255 || voxel_coords.z>255 || voxel_coords.x<0 || voxel_coords.y<0 || voxel_coords.z<0){
            asign_color(pixel,0x00ff00ffu);//VERDE
            return true;
        }

        if(is_voxel_filled(voxel_coords) != 0u){
            
            /*
            if(i==0){
                asign_color(pixel,0x00ff00ffu);
                return false;
            }
            */
            //o_hit_axis = mask; //redundante //usar para pruebas despues

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

        //mask = vec3<bool>(side_distance.x<=mini.x, side_distance.y<=mini.y, side_distance.z<=mini.z);
        mask = vec3<bool>(side_distance<=mini);

        side_distance = side_distance + (vec3<f32>(f32(mask.x),f32(mask.y),f32(mask.z))* rayo.direction * ray_step);

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
    /*
    if(coordenadas.z==101 && 0<coordenadas.y && coordenadas.y<300 && 0<coordenadas.x && coordenadas.x<300) {
        //grid[(x + world.1 * (y +(world.2 * z))) as usize] = 0xff0000ff;
        color = 0xff0000ffu;
    }
    if(coordenadas.z==100 && 10<coordenadas.y && coordenadas.y<50 && 10<coordenadas.x && coordenadas.x<120) {
        //grid[(x + world.1 * (y +(world.2 * z))) as usize] = 0xff0000ff;
        color = 0xff00ffffu;
    }
    */


    if(coordenadas.x <= 5){
        //color -x BLANCO
        color = 0xffffffffu;
    }
    if(coordenadas.x >= 250){
        //color +x NEGRO
        color = 0x000000ffu;
    }
    if(coordenadas.y <= 5){
        //color -y ROJO
        //color = 0xff0000ffu;
        color = hex_color(255f,0f,0f,255f);
    }
    if(coordenadas.y >= 250){
        //color +y AMARILLO
        color = 0xffff00ffu;
    }
    if(coordenadas.z <= 5){
        //color -z CELESTE
        color = 0x00ffffffu;
    }
    if(coordenadas.z >= 250){
        //color +z FUXIA
        color = 0xff00ffffu;
    }
    

    return color;


}


fn asign_color(pixel: vec2<u32>, color: u32) {
    //pixel.x = pixel.x.mod(512.)
    imagen[pixel.x+(512*pixel.y)]= color;

}

fn hex_color(r:f32,g:f32,b:f32,a:f32) -> u32 {

    //?buscar forma de no usar clamp?
    var nr :u32 = u32(clamp(i32(r),0i,255i));
    var ng :u32 = u32(clamp(i32(g),0i,255i));
    var nb :u32 = u32(clamp(i32(b),0i,255i));
    var na :u32 = u32(clamp(i32(a),0i,255i));

    nr = nr*16777216;//16 elevado a 6
    ng = ng*65536;//16 elevado a 4
    nb = nb*256;//16 elevado a 2

    return nr+ng+nb+na;
}

// para comparar un u32 y un f32 se hace f32(variable) que es lo mismo que "variable as f32"