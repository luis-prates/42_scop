#[derive(Default, Clone)]
pub struct ObjLoadOptions {
    pub triangulate: bool,
    pub single_index: bool,
}

#[derive(Default, Clone)]
pub struct ObjMeshData {
    pub positions: Vec<f32>,
    pub normals: Vec<f32>,
    pub texcoords: Vec<f32>,
    pub indices: Vec<u32>,
    pub material_id: Option<usize>,
}

#[derive(Default, Clone)]
pub struct ObjObjectData {
    pub mesh: ObjMeshData,
}

#[derive(Default, Clone)]
pub struct ObjMaterialData {
    pub name: String,
    pub diffuse_texture: Option<String>,
    pub specular_texture: Option<String>,
    pub normal_texture: Option<String>,
}

#[derive(Default, Clone)]
pub struct ObjSceneData {
    pub objects: Vec<ObjObjectData>,
    pub materials: Vec<ObjMaterialData>,
}
