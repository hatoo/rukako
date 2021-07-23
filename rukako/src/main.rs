use std::{borrow::Cow, fs::File, num::NonZeroU64, path::Path};

use image::{png::PngEncoder, ImageEncoder};
use rukako_shader::ShaderConstants;
use wgpu::util::DeviceExt;

const SHADER: &[u8] = include_bytes!(env!("rukako_shader.spv"));

async fn run(width: usize, height: usize, output_png_file_name: impl AsRef<Path>) {
    let instance = wgpu::Instance::new(wgpu::BackendBit::all());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .expect("Failed to find an appropriate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::PUSH_CONSTANTS,
                limits: wgpu::Limits {
                    max_push_constant_size: 256,
                    ..wgpu::Limits::default()
                },
            },
            None,
        )
        .await
        .expect("Failed to create device");

    // Load the shaders from disk
    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::SpirV(Cow::Borrowed(bytemuck::cast_slice(SHADER))),
        flags: wgpu::ShaderFlags::default(),
    });

    /*
    let buffer_dimensions = BufferDimensions::new(width, height);

    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (buffer_dimensions.padded_bytes_per_row * buffer_dimensions.height) as u64,
        usage: wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
        mapped_at_creation: false,
    });

    let texture_extent = wgpu::Extent3d {
        width: buffer_dimensions.width as u32,
        height: buffer_dimensions.height as u32,
        depth_or_array_layers: 1,
    };

    // The render pipeline renders data into this texture
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::COPY_SRC,
        label: None,
    });
    */

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            // XXX - some graphics cards do not support empty bind layout groups, so
            // create a dummy entry.
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                count: None,
                visibility: wgpu::ShaderStage::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    has_dynamic_offset: false,
                    min_binding_size: Some(NonZeroU64::new(1).unwrap()),
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                },
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[wgpu::PushConstantRange {
            stages: wgpu::ShaderStage::COMPUTE,
            range: 0..std::mem::size_of::<ShaderConstants>() as u32,
        }],
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: "main_cs",
    });

    let src: Vec<u8> = vec![0; 4 * 4 * width * height];

    let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: src.len() as wgpu::BufferAddress,
        // Can be read to the CPU, and can be copied from the shader's storage buffer
        usage: wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
        mapped_at_creation: false,
    });

    let storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Collatz Conjecture Input"),
        contents: &src,
        usage: wgpu::BufferUsage::STORAGE
            | wgpu::BufferUsage::COPY_DST
            | wgpu::BufferUsage::COPY_SRC,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: storage_buffer.as_entire_binding(),
        }],
    });

    let push_constants = ShaderConstants {
        width: width as u32,
        height: height as u32,
    };

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&compute_pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.set_push_constants(0, bytemuck::bytes_of(&push_constants));
        cpass.dispatch((width as u32 + 7) / 8, (height as u32 + 7) / 8, 1);
    }

    encoder.copy_buffer_to_buffer(
        &storage_buffer,
        0,
        &readback_buffer,
        0,
        src.len() as wgpu::BufferAddress,
    );

    queue.submit(Some(encoder.finish()));

    let buffer_slice = readback_buffer.slice(..);
    let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);

    device.poll(wgpu::Maintain::Wait);

    if let Ok(()) = buffer_future.await {
        let padded_buffer = buffer_slice.get_mapped_range();

        let png_encoder = PngEncoder::new(File::create(output_png_file_name).unwrap());

        let v4: &[f32] = bytemuck::cast_slice(&padded_buffer[..]);

        let rgba: Vec<u8> = v4
            .iter()
            .map(|f| (256.0 * f.clamp(0.0, 0.999)) as u8)
            .collect();
        png_encoder
            .write_image(
                rgba.as_slice(),
                width as u32,
                height as u32,
                image::ColorType::Rgba8,
            )
            .unwrap();
        drop(padded_buffer);

        readback_buffer.unmap();
    }
}

fn main() {
    env_logger::init();
    pollster::block_on(run(256, 256, "out.png"));
}
