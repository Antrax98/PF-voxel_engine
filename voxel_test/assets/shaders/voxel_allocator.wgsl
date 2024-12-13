/*
alocar los brickmaps dependiendo de lo que diga el command buffers
cambiando lo que aparesca en el brickmap_buffer, y brickmap_data
tambien despues ver como manejar la grid de orden superior
*/

struct Brickmap {
    datos: array<u32,16>,
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

struct BMAlloc {
    bm_idx: u32,
    bm_buffer_idx: u32
}

struct Comando {
    allocar: BMAlloc,
    deallocar: u32,
    datos: Brickmap,
    com: u32
}

@group(0) @binding(0)
var<storage, read_write> var_dat: VarData;

@group(0) @binding(1)
var<storage, read_write> comandos: array<Comando>;

@group(0) @binding(2)
var<storage, read_write> brickgrid: array<u32>;// grid de 256x256x256 Cells POR AHORA

@group(0) @binding(3)
var<storage, read_write> brickmap_data: array<Brickmap>;//brickmaps alocados


@compute @workgroup_size(1,1,1)//64
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {

    let comando = comandos[invocation_id.x];

    //cada invocacion deve manejar solo un comando, de haber menos comandos que invocaciones, las invocaciones sobranter simplemente retornaran antes que nada
    //si no funciona, mover todo a una funcion que debuelva un bool.
    if (var_dat.command_buffer_size<= invocation_id.x){
        return;
    }

    //MODIFICAR ESTO SI ES QUE PUEDEN HABER BRICKMAPS VACIOS EN EL FUTURO (el render tendra que leer el flag y ver que la vandera de estar vacio el true, ojo en que vacio y no estar cargado son distintos)
    if comando.com == 0u {
        brickgrid[comando.allocar.bm_idx]= (comando.allocar.bm_buffer_idx << 8u) + 0u;
        return;
    }

    //hacer esto solo si el brickmap NO ESTA VACIO, si esta vacio es un //TODO (no se escribira el brickmap en el brickmap_buffer)
    //var eux: u32 = 
    brickgrid[comando.allocar.bm_idx]= (comando.allocar.bm_buffer_idx << 8u) + 1u;//AQUI HACER MAGIA INCERTANDO EL BM_BUFFER_IDX junto a los 8 bits de flags

    brickmap_data[comando.allocar.bm_buffer_idx] = comando.datos;


    //8bit flags
    // 0u == vacio == 0000_0000b == 0x00u
    // 1u == cargado == 0000_0001b == 0x01u
    // 2u == no cargado == 0000_0010b == 0x02u
    // 4U == cargando == 0000_0100b == 0x04u

    if (comando.allocar.bm_idx != comando.deallocar){
        //desalocar
        //24 bits de color azul medio oscuro y 8 bits de flag no cargado = 0x355f95u
        var aux: u32 = 0x355f9502u;
        brickgrid[comando.deallocar] = aux;//AQUI PONER PUNTERO DEFAULT EL QUE DICE QUE NO ESTA CARGADO

    }



}