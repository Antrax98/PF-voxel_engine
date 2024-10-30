struct brickmap {
    datos: u32,
}

struct InitData {
    imagen_height: u32,
    imagen_width: u32
}

//este se usara como recipiente para todos los datos unicos necesarios de solo lectura
//obtenibles al iniciar la aplicacion (no se pueden cambiar despues)
@group(0) @binding(0)
var<uniform> data_struct: f32;

@group(0) @binding(1)
var<storage, read_write> variable_data: array<u32>;

//usar color en Hexadecimal
@group(0) @binding(2)
var<storage, read_write> imagen: array<u32>;

@group(0) @binding(3)
var<storage, read_write> brickgrid: array<u32>;

@group(0) @binding(4)
var<storage, read_write> brickmap_data: array<brickmap>;



@compute @workgroup_size(1,1,1)//64
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {

    if (invocation_id.x==0){
        let dum: f32 = data_struct;
        let dim: u32 = variable_data[0];
        //let dom: u32 = imagen[0];
        let dem: u32 = brickgrid[0];
        let dam: u32 = brickmap_data[0].datos;
    }

    imagen[invocation_id.x+(512*invocation_id.y)] = 0x00ffffffu;
    if(variable_data[0]%2< 1 && invocation_id.x<60){
        imagen[invocation_id.x+(512*invocation_id.y)] = 0xffffffffu;
    }
    if(variable_data[0]%2== 1 && invocation_id.x>400){
        imagen[invocation_id.x+(512*invocation_id.y)] = 0xffffff00u;
    }


    //for(var i:u32=0;i<)
}

/*
fn save_color(invocation_id: vec3<u32>, color: u32) {
    return
}
*/