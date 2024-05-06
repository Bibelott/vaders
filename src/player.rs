use crate::{input, sprite::Sprite, Context, Renderer};
use image::io::Reader as ImageReader;
use wgpu::util::DeviceExt;
use winit::keyboard::KeyCode;

const SPEED: f32 = 0.08;

pub struct Player {
    sprite: Sprite,
    texture: wgpu::Texture,
}
impl Player {
    pub fn init(context: &Context, sampler: &wgpu::Sampler, renderer: &Renderer) -> Self {
        let img = ImageReader::open("player.png")
            .unwrap()
            .decode()
            .unwrap()
            .flipv()
            .to_rgba8();
        let texels = bytemuck::cast_slice(img.as_raw());

        let texture = context.device.create_texture_with_data(
            &context.queue,
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: img.width(),
                    height: img.height(),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::MipMajor,
            texels,
        );
        let sprite = Sprite::new(
            nalgebra_glm::vec2(30.0, 30.0),
            nalgebra_glm::vec2(13.0, 8.0),
            &texture,
            &renderer.pipeline.get_bind_group_layout(1),
            sampler,
            context,
        );

        Self { sprite, texture }
    }

    pub fn update(&mut self, context: &Context) {
        if input::is_key_pressed(KeyCode::ArrowLeft) {
            self.sprite
                .move_by(&nalgebra_glm::vec2(-SPEED, 0.0), context);
        }
        if input::is_key_pressed(KeyCode::ArrowRight) {
            self.sprite
                .move_by(&nalgebra_glm::vec2(SPEED, 0.0), context);
        }
    }

    pub fn get_sprite(&self) -> &Sprite {
        &self.sprite
    }
}
