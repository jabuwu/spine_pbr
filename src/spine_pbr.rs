use std::mem::take;

use bevy::{
    ecs::system::{StaticSystemParam, SystemParam},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{Indices, MeshVertexAttribute, MeshVertexBufferLayout},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
            VertexFormat,
        },
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin, Mesh2dHandle},
};
use bevy_spine::{
    materials::{SpineMaterial, SpineMaterialInfo, SpineMaterialPlugin},
    textures::SpineTexture,
    Spine, SpineMesh, SpineMeshState, SpineSet,
};

use crate::cursor::Cursor;

pub const DARK_COLOR_SHADER_POSITION: usize = 10;
pub const DARK_COLOR_ATTRIBUTE: MeshVertexAttribute = MeshVertexAttribute::new(
    "Vertex_DarkColor",
    DARK_COLOR_SHADER_POSITION,
    VertexFormat::Float32x4,
);
pub const ROTATION_SHADER_POSITION: usize = 11;
pub const ROTATION_ATTRIBUTE: MeshVertexAttribute = MeshVertexAttribute::new(
    "Vertex_Rotation",
    ROTATION_SHADER_POSITION,
    VertexFormat::Float32x4,
);

pub struct SpinePbrPlugin;

impl Plugin for SpinePbrPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<SpinePbrMaterial>::default())
            .add_plugin(SpineMaterialPlugin::<SpinePbrMaterial>::default())
            .add_system(spine_pbr_update_meshes.in_set(SpineSet::OnUpdateMesh));
    }
}

#[derive(Component)]
pub struct SpinePbr {
    pub normal_map: Handle<Image>,
}

#[derive(AsBindGroup, TypeUuid, Clone, Default)]
#[uuid = "2e85f9ae-049a-4bb5-9f5d-ebaaa208df60"]
pub struct SpinePbrMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub image: Handle<Image>,

    #[texture(2)]
    #[sampler(3)]
    pub normal_image: Handle<Image>,

    #[uniform(4)]
    pub light_position: Vec4,
}

impl Material2d for SpinePbrMaterial {
    fn vertex_shader() -> ShaderRef {
        "spine_pbr_vert.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "spine_pbr_frag.wgsl".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let mut vertex_attributes = Vec::new();
        vertex_attributes.push(Mesh::ATTRIBUTE_POSITION.at_shader_location(0));
        vertex_attributes.push(Mesh::ATTRIBUTE_NORMAL.at_shader_location(1));
        vertex_attributes.push(Mesh::ATTRIBUTE_UV_0.at_shader_location(2));
        vertex_attributes.push(Mesh::ATTRIBUTE_COLOR.at_shader_location(4));
        vertex_attributes
            .push(DARK_COLOR_ATTRIBUTE.at_shader_location(DARK_COLOR_SHADER_POSITION as u32));
        vertex_attributes
            .push(ROTATION_ATTRIBUTE.at_shader_location(ROTATION_SHADER_POSITION as u32));
        let vertex_buffer_layout = layout.get_layout(&vertex_attributes)?;
        descriptor.vertex.buffers = vec![vertex_buffer_layout];
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

#[derive(SystemParam)]
pub struct SpinePbrMaterialParam<'w, 's> {
    spine_pbr_query: Query<'w, 's, &'static SpinePbr>,
    cursor: Res<'w, Cursor>,
}

impl SpineMaterial for SpinePbrMaterial {
    type Material = Self;
    type Params<'w, 's> = SpinePbrMaterialParam<'w, 's>;

    fn update<'w, 's>(
        material: Option<Self>,
        entity: Entity,
        renderable_data: SpineMaterialInfo,
        params: &StaticSystemParam<Self::Params<'w, 's>>,
    ) -> Option<Self> {
        let SpinePbrMaterialParam {
            spine_pbr_query,
            cursor,
        } = &**params;
        if let Ok(spine_pbr) = spine_pbr_query.get(entity) {
            let mut material = material.unwrap_or_else(|| Self::default());
            material.image = renderable_data.texture;
            material.normal_image = spine_pbr.normal_map.clone();
            material.light_position =
                Vec4::new(cursor.world_position.x, cursor.world_position.y, 0.1, 0.);
            Some(material)
        } else {
            None
        }
    }
}

fn spine_pbr_update_meshes(
    mut spine_query: Query<(&mut Spine, &Children), With<SpinePbr>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_query: Query<(
        Entity,
        &mut SpineMesh,
        &mut Transform,
        Option<&Mesh2dHandle>,
    )>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (mut spine, spine_children) in spine_query.iter_mut() {
        let mut renderables = spine.0.renderables();
        let mut z = 0.;
        let mut renderable_index = 0;
        for child in spine_children.iter() {
            if let Ok((
                spine_mesh_entity,
                mut spine_mesh,
                mut spine_mesh_transform,
                spine_2d_mesh,
            )) = mesh_query.get_mut(*child)
            {
                if !spine_2d_mesh.is_some() {
                    if let Some(mut entity) = commands.get_entity(spine_mesh_entity) {
                        entity.insert(Mesh2dHandle(spine_mesh.handle.clone()));
                    }
                }
                let Some(mesh) = meshes.get_mut(&spine_mesh.handle) else {
                    continue;
                };
                let mut empty = true;
                'render: {
                    let Some(renderable) = renderables.get_mut(renderable_index) else {
                        break 'render;
                    };
                    let slot_index = Some(renderable.slot_index as usize);
                    let attachment_renderer_object = renderable.attachment_renderer_object;
                    let indices = take(&mut renderable.indices);
                    let vertices = take(&mut renderable.vertices);
                    let uvs = take(&mut renderable.uvs);
                    let colors = vec![
                        [
                            renderable.color.r,
                            renderable.color.g,
                            renderable.color.b,
                            renderable.color.a
                        ];
                        vertices.len()
                    ];
                    let dark_colors = vec![
                        [
                            renderable.dark_color.r,
                            renderable.dark_color.g,
                            renderable.dark_color.b,
                            renderable.dark_color.a
                        ];
                        vertices.len()
                    ];
                    let mut rotations_count = vec![0.; vertices.len()];
                    let mut rotations = vec![[0., 0., 0., 1.]; vertices.len()];
                    for i in (0..indices.len()).step_by(3) {
                        let (i1, i2, i3) = (
                            indices[i] as usize,
                            indices[i + 1] as usize,
                            indices[i + 2] as usize,
                        );
                        let tri_verts = [vertices[i1], vertices[i2], vertices[i3]];
                        let tri_uvs = [uvs[i1], uvs[i2], uvs[i3]];

                        let angle1 = (Vec2::from(tri_verts[0]) - Vec2::from(tri_verts[1]))
                            .normalize()
                            * Vec2::new(1., -1.);
                        let angle2 = (Vec2::from(tri_uvs[0]) - Vec2::from(tri_uvs[1])).normalize();
                        let rotation = angle2.angle_between(angle1);

                        fn angle_lerp(old_angle: f32, new_angle: f32, x: f32) -> f32 {
                            old_angle
                                + Vec2::from_angle(old_angle)
                                    .angle_between(Vec2::from_angle(new_angle))
                                    * x
                        }

                        rotations_count[i1] += 1.;
                        rotations_count[i2] += 1.;
                        rotations_count[i3] += 1.;
                        rotations[i1][0] =
                            angle_lerp(rotations[i1][0], rotation, 1. / rotations_count[i1]);
                        rotations[i2][0] =
                            angle_lerp(rotations[i2][0], rotation, 1. / rotations_count[i2]);
                        rotations[i3][0] =
                            angle_lerp(rotations[i3][0], rotation, 1. / rotations_count[i3]);
                    }
                    for rotation in rotations.iter_mut() {
                        let rotation_matrix = Mat4::from_rotation_z(rotation[0]);
                        rotation[0] = rotation_matrix.x_axis[0];
                        rotation[1] = rotation_matrix.y_axis[0];
                        rotation[2] = rotation_matrix.x_axis[1];
                        rotation[3] = rotation_matrix.y_axis[1];
                    }
                    let blend_mode = renderable.blend_mode;
                    let premultiplied_alpha = renderable.premultiplied_alpha;
                    let Some(attachment_render_object) = attachment_renderer_object else {
                        break 'render;
                    };
                    let spine_texture =
                        unsafe { &mut *(attachment_render_object as *mut SpineTexture) };
                    let texture_path = spine_texture.0.clone();
                    let mut normals = vec![];
                    for _ in 0..vertices.len() {
                        normals.push([0., 0., 0.]);
                    }
                    mesh.set_indices(Some(Indices::U16(indices)));
                    mesh.insert_attribute(
                        MeshVertexAttribute::new("Vertex_Position", 0, VertexFormat::Float32x2),
                        vertices,
                    );
                    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
                    mesh.insert_attribute(DARK_COLOR_ATTRIBUTE, dark_colors);
                    mesh.insert_attribute(ROTATION_ATTRIBUTE, rotations);
                    spine_mesh.state = SpineMeshState::Renderable {
                        info: SpineMaterialInfo {
                            slot_index,
                            texture: asset_server.load(texture_path.as_str()),
                            blend_mode,
                            premultiplied_alpha,
                        },
                    };
                    spine_mesh_transform.translation.z = z;
                    z += 0.001;
                    empty = false;
                }
                if empty {
                    spine_mesh.state = SpineMeshState::Empty;
                    empty_mesh(mesh);
                }
                renderable_index += 1;
            }
        }
    }
}

fn empty_mesh(mesh: &mut Mesh) {
    let indices = Indices::U32(vec![]);

    let positions: Vec<[f32; 3]> = vec![];
    let normals: Vec<[f32; 3]> = vec![];
    let uvs: Vec<[f32; 2]> = vec![];
    let colors: Vec<[f32; 4]> = vec![];
    let dark_colors: Vec<[f32; 4]> = vec![];
    let rotations: Vec<[f32; 4]> = vec![];

    mesh.set_indices(Some(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_attribute(DARK_COLOR_ATTRIBUTE, dark_colors);
    mesh.insert_attribute(ROTATION_ATTRIBUTE, rotations);
}
