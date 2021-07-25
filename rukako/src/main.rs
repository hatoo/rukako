use std::{borrow::Cow, fs::File, num::NonZeroU64, path::Path};

use image::{png::PngEncoder, ImageEncoder};
use rand::prelude::StdRng;
use rand::prelude::*;
use rukako_shader::{
    pod::{EnumMaterialPod, Lambertian, Sphere},
    ShaderConstants, NUM_THREADS_X, NUM_THREADS_Y,
};
use spirv_std::glam::{vec3, Vec3};
use wgpu::util::DeviceExt;

const SHADER: &[u8] = include_bytes!(env!("rukako_shader.spv"));

fn random_scene() -> Vec<Sphere> {
    let mut rng = StdRng::from_entropy();

    let mut world = Vec::new();

    world.push(Sphere::new(
        vec3(0.0, -1000.0, 0.0),
        1000.0,
        EnumMaterialPod::new_lambertian(vec3(0.5, 0.5, 0.5)),
    ));

    for a in -11..11 {
        for b in -11..11 {
            let center = vec3(
                a as f32 + 0.9 * rng.gen::<f32>(),
                0.2,
                b as f32 + 0.9 * rng.gen::<f32>(),
            );

            let choose_mat: f32 = rng.gen();

            if (center - vec3(4.0, 0.2, 0.0)).length() > 0.9 {
                match choose_mat {
                    x if x < 0.8 => {
                        let albedo = vec3(rng.gen(), rng.gen(), rng.gen())
                            * vec3(rng.gen(), rng.gen(), rng.gen());

                        world.push(Sphere::new(
                            center,
                            0.3,
                            EnumMaterialPod::new_lambertian(albedo),
                        ));
                    }
                    x if x < 0.95 => {
                        let albedo = vec3(
                            rng.gen_range(0.5..1.0),
                            rng.gen_range(0.5..1.0),
                            rng.gen_range(0.5..1.0),
                        );
                        let fuzz = rng.gen_range(0.0..0.5);

                        world.push(Sphere::new(
                            center,
                            0.2,
                            EnumMaterialPod::new_metal(albedo, fuzz),
                        ));
                    }
                    _ => world.push(Sphere::new(
                        center,
                        0.2,
                        EnumMaterialPod::new_dielectric(1.5),
                    )),
                }
            }
        }
    }

    world.push(Sphere::new(
        vec3(0.0, 1.0, 0.0),
        1.0,
        EnumMaterialPod::new_dielectric(1.5),
    ));
    world.push(Sphere::new(
        vec3(-4.0, 1.0, 0.0),
        1.0,
        EnumMaterialPod::new_lambertian(vec3(0.4, 0.2, 0.1)),
    ));
    world.push(Sphere::new(
        vec3(4.0, 1.0, 0.0),
        1.0,
        EnumMaterialPod::new_metal(vec3(0.7, 0.6, 0.5), 0.0),
    ));

    world
}

async fn run(
    width: usize,
    height: usize,
    n_samples: usize,
    output_png_file_name: impl AsRef<Path>,
) {
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

    let world = random_scene();

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                count: None,
                visibility: wgpu::ShaderStage::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    has_dynamic_offset: false,
                    min_binding_size: Some(NonZeroU64::new(1).unwrap()),
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                },
            },
            // XXX - some graphics cards do not support empty bind layout groups, so
            // create a dummy entry.
            wgpu::BindGroupLayoutEntry {
                binding: 1,
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
        label: Some("Output Image"),
        contents: &src,
        usage: wgpu::BufferUsage::STORAGE
            | wgpu::BufferUsage::COPY_DST
            | wgpu::BufferUsage::COPY_SRC,
    });

    let world_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("world"),
        contents: bytemuck::cast_slice(world.as_slice()),
        usage: wgpu::BufferUsage::STORAGE
            // | wgpu::BufferUsage::COPY_DST
            // | wgpu::BufferUsage::COPY_SRC,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: world_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: storage_buffer.as_entire_binding(),
            },
        ],
    });

    let mut rng = StdRng::from_entropy();

    let mut push_constants = ShaderConstants {
        width: width as u32,
        height: height as u32,
        world_len: world.len() as u32,
        seed: rng.gen(),
    };

    for i in 0..n_samples {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&compute_pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);

            push_constants.seed = rng.gen();
            cpass.set_push_constants(0, bytemuck::bytes_of(&push_constants));
            cpass.dispatch(
                (width as u32 + NUM_THREADS_X - 1) / NUM_THREADS_X,
                (height as u32 + NUM_THREADS_Y - 1) / NUM_THREADS_Y,
                1,
            );
        }
        queue.submit(Some(encoder.finish()));
        device.poll(wgpu::Maintain::Wait);
        eprint!("\rSaamples: {} / {} ", i + 1, n_samples);
    }
    eprint!("\nDone");

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
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
        dbg!(v4[0]);

        let scale = 1.0 / n_samples as f32;

        let rgba: Vec<u8> = v4
            .iter()
            .map(|f| (256.0 * (f * scale).sqrt().clamp(0.0, 0.999)) as u8)
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
    pollster::block_on(run(1200, 800, 500, "out.png"));
}
