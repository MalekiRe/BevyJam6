/// ----------------------------------------------
/// material example
/// how to add a custom material
/// ----------------------------------------------
use bevy::{
	core_pipeline::bloom::Bloom, diagnostic::DiagnosticsStore,
	image::ImageSamplerDescriptor, prelude::*, render::render_resource::AsBindGroup,
};
use bevy_enoki::{EnokiPlugin, prelude::*};

#[derive(AsBindGroup, Asset, TypePath, Clone, Default)]
pub struct FireParticleMaterial {
	#[texture(0)]
	#[sampler(1)]
	pub(crate) texture: Handle<Image>,
}

impl Particle2dMaterial for FireParticleMaterial {
	fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
		"shaders/custom_material.wgsl".into()
	}
}
