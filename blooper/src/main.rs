#![deny(warnings)]
mod cs;
use arrayvec::ArrayVec;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::mem::size_of;
use std::net::TcpListener;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use tungstenite::accept;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBuffer;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;
use vulkano::pipeline::ComputePipeline;
use vulkano::sync::GpuFuture;
//#[allow(non_camel_case_types)]
//type pid = usize;
const MAX_COLLISION: usize = 1024;
const MAX_PARTICLES_COUNT: usize = 1024 * 64;
const MAX_GRID_SIZE: usize = 128;
#[derive(Clone, Copy)]
#[allow(dead_code)]
struct Particle {
    is_active: u32,
    d: f32,
    x: f32,
    y: f32,
    x_before: f32,
    y_before: f32,
    mass: f32,
    grid_x: u32,
    grid_y: u32,
    collisions_count: u32,
}
#[derive(Clone, Serialize, Deserialize)]
struct Configuration {
    constants: Constants,
    initial_max_speed_per_s: f32,
    multiplier: f32,
    port: u32,
    address: String,
    initial_particle_count: usize,
    gpu_id: usize,
    serialize_unactive_particles: bool,
    update_client_data: bool,
    show_gpu_supported_features: bool,
}
#[derive(Copy, Clone, Serialize, Deserialize)]
struct Constants {
    width: f32,
    height: f32,
    delta_time_s: f32,
    grid_size: u32,
    default_diameter: f32,
    world_size: f32,
    collision_push_rate: f32,
    default_mass: f32,
    gravity: Vec2,
}
#[derive(Copy, Clone, Serialize, Deserialize)]
struct Vec2 {
    x: f32,
    y: f32,
}
#[allow(dead_code)]
struct PushConstants {
    gravity: Vec2,
    i_source: u32,
    i_target: u32,
    width: f32,
    height: f32,
    grid_size: u32,
    delta_time_s: f32,
    collision_push_rate: f32,
}
#[derive(Serialize, Deserialize)]
struct ParticleClientData {
    x: f32,
    y: f32,
    d: f32,
    a: bool,
}
#[derive(Serialize, Deserialize)]
struct ClientData {
    particles: Vec<ParticleClientData>,
    constants: Constants,
    tick: usize,
}
#[derive(Copy, Clone)]
struct CollisionCell {
    count: u32,
    pids: [u32; MAX_COLLISION],
}

struct Data {
    particles: [[Particle; 2]; MAX_PARTICLES_COUNT],
    collision_grid: [[CollisionCell; MAX_GRID_SIZE]; MAX_GRID_SIZE],
}

fn test_particles() {
    println!("Blooper");
    let configuration_str: String = env::var("blooper_configuration")
        .unwrap()
        .replace("\\\"", "\"");
    let configuration: Configuration = serde_json::from_str(&configuration_str).unwrap();
    let instance =
        Instance::new(None, &InstanceExtensions::none(), None).expect("failed to create instance");
    let physical_devices = PhysicalDevice::enumerate(&instance);
    println!("Available devices");
    for (i, physical_device) in physical_devices.clone().enumerate() {
        println!("  {} - {}", i, physical_device.name());
        if configuration.show_gpu_supported_features {
            println!("    {:?}", physical_device.supported_features());
        }
    }
    let physical_device_id = configuration.gpu_id;
    let physical_device = physical_devices.collect::<Vec<PhysicalDevice>>()[physical_device_id];
    println!("using physical device {}", physical_device.name());
    println!("Families");
    for family in physical_device.queue_families() {
        println!("  #{:?}", family.id());
        println!("    queues: {:?}", family.queues_count());
        println!("    supports_compute: {:?}", family.supports_compute());
    }
    let family_id = 0;
    println!("using family #{}", family_id);
    let extensions = DeviceExtensions {
        khr_storage_buffer_storage_class: true,
        ..DeviceExtensions::none()
    };
    let features = Features { ..Features::none() };
    let (device, mut queues) = {
        Device::new(
            physical_device,
            &features,
            &extensions,
            [(physical_device.queue_families().next().unwrap(), 0.5)]
                .iter()
                .cloned(),
        )
        .expect("failed to create device")
    };
    println!("loaded extensions: {:#?}", device.loaded_extensions());
    let queue = queues.next().unwrap();
    let local_size_x = 64;
    let work_groups_count: u32 = MAX_PARTICLES_COUNT as u32 / local_size_x;
    let particles_size = size_of::<[[Particle; 2]; MAX_PARTICLES_COUNT]>();
    let collision_grid_size = size_of::<[[CollisionCell; MAX_GRID_SIZE]; MAX_GRID_SIZE]>();
    let stack_size = particles_size * 4 + collision_grid_size * 4 + 100_000;
    println!("size: {}", stack_size);
    let thread_builder = thread::Builder::new();
    let handler = thread_builder
        .stack_size(stack_size)
        .spawn(move || {
            let constants = Constants {
                world_size: configuration.constants.world_size * configuration.multiplier,
                width: configuration.constants.width * configuration.multiplier,
                height: configuration.constants.height * configuration.multiplier,
                default_diameter: configuration.constants.default_diameter
                    * configuration.multiplier,
                ..configuration.constants
            };
            let particles: [[Particle; 2]; MAX_PARTICLES_COUNT] = get_random_particles(
                &configuration,
                MAX_PARTICLES_COUNT,
                configuration.initial_max_speed_per_s,
            )
            .into_iter()
            .collect::<ArrayVec<_>>()
            .into_inner()
            .unwrap_or_else(|_| unreachable!());
            let collision_cell = CollisionCell {
                count: 0,
                pids: [0; MAX_COLLISION],
            };
            let collision_grid: [[CollisionCell; MAX_GRID_SIZE]; MAX_GRID_SIZE] =
                [[collision_cell; MAX_GRID_SIZE]; MAX_GRID_SIZE];
            let data = Data {
                particles: particles,
                collision_grid: collision_grid,
            };
            let host_cached = false;
            let buffer = CpuAccessibleBuffer::from_data(
                device.clone(),
                BufferUsage::all(),
                host_cached,
                data,
            )
            .expect("failed to create buffer");
            let shader = cs::Shader::load(device.clone()).expect("failed to create shader module");
            let compute_pipeline = Arc::new(
                ComputePipeline::new(device.clone(), &shader.main_entry_point(), &())
                    .expect("failed to create compute pipeline"),
            );
            let layout = compute_pipeline.layout().descriptor_set_layout(0).unwrap();
            let set = Arc::new(
                PersistentDescriptorSet::start(layout.clone())
                    .add_buffer(buffer.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            );
            let mut tick = 0;
            let client_data = "".to_string();
            let client_data_lock = Arc::new(RwLock::new(client_data));
            //
            // thread client
            //
            {
                let client_data_lock_clone = Arc::clone(&client_data_lock);
                let configuration_clone = configuration.clone();
                thread::spawn(move || {
                    handle_websocket(client_data_lock_clone, configuration_clone);
                });
            }
            let client_data_lock_clone = Arc::clone(&client_data_lock);
            let mut durations: Vec<u128> = Vec::new();
            loop {
                let start_time = SystemTime::now();
                let i_source: usize = tick % 2;
                let i_target: usize = (i_source as usize + 1) % 2;
                // write

                {
                    let mut buffer_write = buffer.write().unwrap();
                    for i in 0..constants.grid_size as usize {
                        for j in 0..constants.grid_size as usize {
                            buffer_write.collision_grid[i][j].count = 0;
                        }
                    }
                    let l = buffer_write.particles.len();
                    for pid in 0..l {
                        let p = buffer_write.particles[pid][i_source];
                        if p.is_active == 0 {
                            continue;
                        }
                        let i = p.grid_x as usize;
                        let j = p.grid_y as usize;
                        let c = { buffer_write.collision_grid[i][j].count as usize };
                        buffer_write.collision_grid[i][j].pids[c] = pid as u32;
                        buffer_write.collision_grid[i][j].count += 1;
                    }
                }
                //for _ in 0..3 {
                // gpu compute
                {
                    let push_constants = PushConstants {
                        i_source: i_source as u32,
                        i_target: i_target as u32,
                        grid_size: constants.grid_size,
                        width: constants.width,
                        height: constants.height,
                        delta_time_s: constants.delta_time_s,
                        collision_push_rate: constants.collision_push_rate,
                        gravity: constants.gravity,
                    };
                    let mut builder =
                        AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap();
                    builder
                        .dispatch(
                            [work_groups_count, 1, 1],
                            compute_pipeline.clone(),
                            set.clone(),
                            push_constants,
                        )
                        .unwrap();
                    let command_buffer = builder.build().unwrap();
                    let finished = command_buffer.execute(queue.clone()).unwrap();
                    finished
                        .then_signal_fence_and_flush()
                        .unwrap()
                        .wait(None)
                        .unwrap();
                }
                tick += 1;
                // write client data
                if configuration.update_client_data {
                    let buffer_read = buffer.read().unwrap();
                    let mut particles_client: Vec<ParticleClientData> = Vec::new();
                    for p in buffer_read.particles.iter() {
                        if p[i_target].is_active == 1 || configuration.serialize_unactive_particles
                        {
                            particles_client.push(ParticleClientData {
                                x: p[i_target].x,
                                y: p[i_target].y,
                                d: p[i_target].d,
                                a: p[i_target].is_active == 1,
                            });
                        }
                    }
                    let client_data = ClientData {
                        constants: constants,
                        particles: particles_client,
                        tick: tick,
                    };
                    *(client_data_lock_clone.write().unwrap()) =
                        serde_json::to_string(&client_data).unwrap().to_string();
                }
                durations.push(
                    SystemTime::now()
                        .duration_since(start_time)
                        .unwrap()
                        .as_millis(),
                );
                for _ in 100..durations.len() {
                    durations.remove(0);
                }
                if tick % 100 == 0 {
                    println!("#{}", tick);
                    println!(
                        "  particles[0].x_: {}",
                        buffer.read().unwrap().particles[0][i_target].x_before
                    );
                    println!(
                        "  particles[0].x:  {}",
                        buffer.read().unwrap().particles[0][i_target].x
                    );
                    let average_duration = mean_u128(&durations);
                    println!("  average_duration:       {}ms", average_duration);
                }
            }
        })
        .unwrap();
    handler.join().unwrap();
}
fn main() {
    test_particles();
}
fn get_random_particles(
    configuration: &Configuration,
    count: usize,
    max_speed_per_sec: f32,
) -> Vec<[Particle; 2]> {
    let mut particles = Vec::new();
    let mut rng = rand::thread_rng();
    let constants = configuration.constants;
    for i in 0..count {
        let x = rng.gen::<f32>() * constants.width;
        let y = rng.gen::<f32>() * constants.height;
        let max_speed_per_tick = max_speed_per_sec * constants.delta_time_s;
        let particle = Particle {
            x: x,
            y: y,
            x_before: x + rng.gen::<f32>() * max_speed_per_tick - max_speed_per_tick * 0.5,
            y_before: y + rng.gen::<f32>() * max_speed_per_tick - max_speed_per_tick * 0.5,
            grid_x: ((x / constants.width * constants.grid_size as f32).abs() as u32)
                .max(0)
                .min(constants.grid_size - 1),
            grid_y: ((y / constants.height * constants.grid_size as f32).abs() as u32)
                .max(0)
                .min(constants.grid_size - 1),
            collisions_count: 0,
            d: constants.default_diameter,
            mass: constants.default_mass,
            is_active: if i < configuration.initial_particle_count {
                1
            } else {
                0
            },
        };
        particles.push([particle, particle]);
    }
    return particles;
}
fn handle_websocket(
    client_data_lock: std::sync::Arc<std::sync::RwLock<String>>,
    configuration: Configuration,
) {
    let address = configuration.address;
    let port = configuration.port;
    let host = format!("{}:{}", address, port);
    let server = TcpListener::bind(host.to_owned()).unwrap();
    println!("server started");
    for stream in server.incoming() {
        println!("incoming");
        let client_data_lock_clone = Arc::clone(&client_data_lock);
        thread::spawn(move || {
            let mut websocket = accept(stream.unwrap()).unwrap();
            let message = websocket.read_message().unwrap();
            println!("message: {}", message);
            if message == tungstenite::Message::Text("server_to_client".to_string()) {
                loop {
                    {
                        let message_write = tungstenite::Message::Text(
                            client_data_lock_clone.read().unwrap().to_string(),
                        );
                        match websocket.write_message(message_write) {
                            Ok(_) => {
                                // Do nothing
                            }
                            Err(error) => {
                                println!("error writer socket: {}", error);
                                break;
                            }
                        }
                    }
                    thread::sleep(Duration::from_millis(10));
                }
            } else if message == tungstenite::Message::Text("latency_checker".to_string()) {
                loop {
                    match websocket.read_message() {
                        Ok(message) => {
                            if message == tungstenite::Message::Text("check".to_string()) {
                                websocket
                                    .write_message(tungstenite::Message::Text(
                                        "check_back".to_string(),
                                    ))
                                    .unwrap();
                            } else {
                                println!("message not handled: {}", message);
                            }
                        }
                        Err(error) => {
                            println!("error: {}", error);
                            break;
                        }
                    }
                }
            } else if message == tungstenite::Message::Text("writer".to_string()) {
                loop {
                    match websocket.read_message() {
                        Ok(message) => {
                            println!("message not handled: {}", message);
                        }
                        Err(error) => {
                            println!("error: {}", error);
                            break;
                        }
                    }
                }
            } else {
                println!("starting message not handled: {}", message);
            }
        });
    }
}
fn mean_u128(v: &Vec<u128>) -> u128 {
    let sum: u128 = Iterator::sum(v.iter());
    if v.len() > 0 {
        sum / (v.len() as u128)
    } else {
        0
    }
}
