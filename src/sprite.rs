use wgpu::util::DeviceExt;

pub struct Sprite {
    model_mat: nalgebra_glm::Mat4,
    model_buf: wgpu::Buffer,
    texture_view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
}

impl Sprite {
    pub fn new(
        pos: nalgebra_glm::Vec2,
        size: nalgebra_glm::Vec2,
        texture: &wgpu::Texture,
        bind_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        context: &crate::Context,
    ) -> Self {
        let pos = nalgebra_glm::vec2_to_vec3(&pos);
        let mut size = nalgebra_glm::vec2_to_vec3(&size);
        size[2] = 1.0;
        let mut model_mat = nalgebra_glm::identity();
        model_mat = nalgebra_glm::translate(&model_mat, &pos);
        model_mat = nalgebra_glm::scale(&model_mat, &size);

        let model_buf = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice((&model_mat).into()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            ..Default::default()
        });

        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: bind_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            model_buf.as_entire_buffer_binding(),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
            });

        Self {
            model_mat,
            model_buf,
            texture_view,
            bind_group,
        }
    }

    pub fn move_by(&mut self, v: &nalgebra_glm::Vec2, context: &crate::Context) {
        let v = nalgebra_glm::vec2_to_vec3(v);
        self.model_mat = nalgebra_glm::translate(&self.model_mat, &v);
        context.queue.write_buffer(
            &self.model_buf,
            0,
            bytemuck::cast_slice((&self.model_mat).into()),
        );
    }

    pub fn get_view(&self) -> &wgpu::TextureView {
        &self.texture_view
    }

    pub fn get_buf(&self) -> &wgpu::Buffer {
        &self.model_buf
    }

    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
