use super::*;

pub struct WirePlugin;

impl Plugin for WirePlugin {
    fn build(&self, app: &mut App) {
        use bevy::asset::embedded_asset;
        embedded_asset!(app, "../assets/wire.wgsl");
        app.add_plugins(MaterialPlugin::<Wire>::default());
    }
}

#[derive(Asset, TypePath, bevy::render::render_resource::AsBindGroup, Clone)]
#[bind_group_data(Key)]
pub struct Wire {
    pub selected: bool,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct Key {
    selected: bool,
}

impl From<&Wire> for Key {
    fn from(material: &Wire) -> Self {
        Self {
            selected: material.selected,
        }
    }
}

impl Material for Wire {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "embedded://stove/../assets/wire.wgsl".into()
    }
    fn specialize(
        _: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _: &bevy::render::mesh::MeshVertexBufferLayout,
        key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        if key.bind_group_data.selected {
            let fragment = descriptor.fragment.as_mut().unwrap();
            fragment.shader_defs.push("SELECTED".into())
        }
        Ok(())
    }
}
