use super::*;

pub struct UnlitPlugin;

impl Plugin for UnlitPlugin {
    fn build(&self, app: &mut App) {
        use bevy::asset::embedded_asset;
        embedded_asset!(app, "unlit.wgsl");
        app.add_plugins(MaterialPlugin::<Unlit>::default());
    }
}

#[derive(bevy::render::render_resource::AsBindGroup, Asset, Reflect, Clone)]
pub struct Unlit {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
}

impl Material for Unlit {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "embedded://stove/unlit.wgsl".into()
    }
}
