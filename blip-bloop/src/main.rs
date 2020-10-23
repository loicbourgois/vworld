use std::time::Duration;
use rapier2d::dynamics::{JointSet, RigidBodySet, IntegrationParameters};
use rapier2d::geometry::{BroadPhase, NarrowPhase, ColliderSet};
use rapier2d::pipeline::PhysicsPipeline;
use std::time::SystemTime;
use rapier2d::na::{Vector2, Isometry2};
use rapier2d::dynamics::{RigidBodyBuilder, BodyStatus};
use rapier2d::geometry::{ColliderBuilder, Shape, Ball};
use rand::prelude::*;
use std::thread;
use std::env;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use ::uuid::Uuid as uuid_maker;
use serde::{Deserialize, Serialize};
use tungstenite::server::accept;
use std::net::TcpListener;
use rapier2d::dynamics::{FixedJoint, BallJoint};
use rapier2d::dynamics::JointParams;
use rapier2d::math::Point;
#[allow(non_camel_case_types)]
type uuid = u128;
#[allow(non_camel_case_types)]
type puuid = uuid;
#[derive(Serialize, Deserialize)]
pub struct ClientData {
    particles: HashMap<puuid, ParticleClientData>,
    width: f32,
    height: f32,
    step: u32,
    links: Vec<[String;2]>,
}
#[derive(Serialize, Deserialize)]
pub struct ParticleClientData {
    x: f32,
    y: f32,
    d: f32,
}
pub struct Data {
    width: f32,
    height: f32,
    step: u32,
    bodies: RigidBodySet,
    diameter: f32,
    count: usize,
    joint_count: usize,
    links: Vec<[puuid;2]>,
}
fn main() {
    println!("Blip Bloop");
    let address: String = env::var("blip_bloop_address").unwrap();
    let port: String = match env::var("PORT") {
        Ok(port) => {
            println!("using env.PORT");
            port
        },
        Err(error) => {
            println!("error getting env.PORT: {:?}", error);
            println!("using blip_bloop_port instead");
            env::var("blip_bloop_port").unwrap()
        }
    };
    // let chunk_configuration_str: String = env::var("vworld_chunk_configuration").unwrap().replace("\\\"", "\"");
    let host = format!("{}:{}", address, port);
    let server = TcpListener::bind(host.to_owned()).unwrap();
    let mut pipeline = PhysicsPipeline::new();
    let gravity = Vector2::new(0.0, -0.0000000);
    let mut integration_parameters = IntegrationParameters::default();
    integration_parameters.set_dt(0.01);
    let mut broad_phase = BroadPhase::new();
    let mut narrow_phase = NarrowPhase::new();
    //let mut bodies = ;
    let mut colliders = ColliderSet::new();
    let mut joints = JointSet::new();
    // We ignore contact events for now.
    let event_handler = ();
    let mut data = Data {
        width: 100.0,
        height: 100.0,
        step: 0,
        bodies: RigidBodySet::new(),
        diameter: 1.0,
        count: 1000,
        joint_count: 200,
        links: Vec::new(),
    };
    let mut rng = rand::thread_rng();
    let mut body_handles = HashMap::new();
    let mut puuids = Vec::new();
    for _ in 0..data.count {
        let x = rng.gen::<f32>() * data.width;
        let y = rng.gen::<f32>() * data.height;
        let body = RigidBodyBuilder::new(BodyStatus::Dynamic)
            .translation(x, y)
            .can_sleep(false)
            .build();
        let body_handle = data.bodies.insert(body);
        let puuid = uuid_maker::new_v4().as_u128();
        body_handles.insert(puuid, body_handle);
        let collider = ColliderBuilder::new(Shape::Ball(Ball::new(data.diameter * 0.5))).build();
        let collider_handle = colliders.insert(collider, body_handle, &mut data.bodies);
        puuids.push(puuid);
    }

    let body_a = RigidBodyBuilder::new(BodyStatus::Static)
        .translation(data.width * 0.5, data.height * 0.5)
        .can_sleep(false)
        .build();
    let body_handle_a = data.bodies.insert(body_a);
    let puuid_a = uuid_maker::new_v4().as_u128();
    body_handles.insert(puuid_a, body_handle_a);
    puuids.push(puuid_a);

    let collider_a = ColliderBuilder::new(Shape::Ball(Ball::new(data.diameter* 0.5))).build();
    let collider_handle_a = colliders.insert(collider_a, body_handle_a, &mut data.bodies);


    let body_b = RigidBodyBuilder::new(BodyStatus::Dynamic)
        .translation(data.width*0.5, data.height*0.5 + data.diameter * 5.0)
        .can_sleep(false)
        .build();
    let body_handle_b = data.bodies.insert(body_b);
    let puuid_b = uuid_maker::new_v4().as_u128();
    body_handles.insert(puuid_b, body_handle_b);
    puuids.push(puuid_b);
    let collider_b = ColliderBuilder::new(Shape::Ball(Ball::new(data.diameter* 0.5))).build();
    let collider_handle_b = colliders.insert(collider_b, body_handle_b, &mut data.bodies);

    let joint_params_ab = BallJoint::new(
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
    );
    let joint_handle_ab = joints.insert(&mut data.bodies, body_handle_a, body_handle_b, joint_params_ab);
    data.links.push([puuid_a, puuid_b]);
    match joints.get(joint_handle_ab).unwrap().params {
        JointParams::BallJoint(joint_params) => {
            println!("joint_ab: {:#?}",  joint_params.impulse);
        }
        _ => {}
    }


    let body_c = RigidBodyBuilder::new(BodyStatus::Dynamic)
        .translation(data.width*0.5 + data.diameter * 5.0, data.height*0.5)
        .can_sleep(false)
        .build();
    let body_handle_c = data.bodies.insert(body_c);
    let puuid_c = uuid_maker::new_v4().as_u128();
    body_handles.insert(puuid_c, body_handle_c);
    puuids.push(puuid_c);
    let collider_c = ColliderBuilder::new(Shape::Ball(Ball::new(data.diameter* 0.5))).build();
    let collider_handle_c = colliders.insert(collider_c, body_handle_c, &mut data.bodies);
    let joint_params_ac = BallJoint::new(
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
    );
    let joint_handle_ac = joints.insert(&mut data.bodies, body_handle_a, body_handle_c, joint_params_ac);
    data.links.push([puuid_a, puuid_c]);
    match joints.get(joint_handle_ac).unwrap().params {
        JointParams::FixedJoint(joint_params) => {
            println!("joint_ac: {:#?}",  joint_params.impulse);
        }
        _ => {}
    }


    for _ in 0..data.joint_count {
        let puuid_id_a = (rng.gen::<f32>() * data.count as f32) as usize;
        let puuid_id_b = (rng.gen::<f32>() * data.count as f32) as usize;
        let puuid_a = puuids[puuid_id_a];
        let puuid_b = puuids[puuid_id_b];
        let body_handle_a = body_handles.get(&puuid_a).unwrap();
        let body_handle_b = body_handles.get(&puuid_b).unwrap();
        let joint_params = BallJoint::new(
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
        );
        let joint_handle = joints.insert(&mut data.bodies, *body_handle_a, *body_handle_b, joint_params);
        data.links.push([puuid_a, puuid_b]);
    }




    let client_data_str = "".to_string();
    let client_data_str_lock = Arc::new(RwLock::new(client_data_str));
    let data_lock = Arc::new(RwLock::new(data));
    let body_handles_lock = Arc::new(RwLock::new(body_handles));
    // Serializer thread
    {
        let client_data_str_lock_clone = Arc::clone(&client_data_str_lock);
        let data_lock_clone = Arc::clone(&data_lock);
        let body_handles_lock_clone = Arc::clone(&body_handles_lock);
        thread::spawn(move || {
            loop {
                let start = SystemTime::now();
                let mut client_data = ClientData {
                    particles: HashMap::new(),
                    height: 0.0,
                    width: 0.0,
                    step: 0,
                    links: Vec::new(),
                };
                {
                    let body_handles_read = &*body_handles_lock_clone.read().unwrap();
                    let data_read = &*data_lock_clone.read().unwrap();
                    for (puuid, body_handle) in body_handles_read {
                        let body = data_read.bodies.get(*body_handle).unwrap();
                        client_data.particles.insert(*puuid, ParticleClientData {
                            x: body.position.translation.x,
                            y: body.position.translation.y,
                            d: data_read.diameter,
                        });
                    }
                    client_data.height = data_read.height;
                    client_data.width = data_read.width;
                    client_data.step = data_read.step;
                    for link in &data_read.links {
                        client_data.links.push([
                            format!("{}", link[0]),
                            format!("{}", link[1])
                        ])
                    }
                }
                *client_data_str_lock_clone.write().unwrap() = serde_json::to_string(&client_data).unwrap().to_string();
                let duration_ms = SystemTime::now().duration_since(start).unwrap().as_millis();
                //println!("json: {}ms", duration_ms);
                thread::sleep(Duration::from_millis(10));
            }
        });
    }
    // Websocket threads
    thread::spawn(move || {
        for stream in server.incoming() {
            println!("new stream");
            let client_data_str_lock_clone = Arc::clone(&client_data_str_lock);
            thread::spawn (move || {
                let mut websocket = accept(stream.unwrap()).unwrap();
                let message = websocket.read_message().unwrap();
                if message == tungstenite::Message::Text("client_reader".to_string()) {
                    loop {
                        {
                            let message_write = tungstenite::Message::Text(client_data_str_lock_clone.read().unwrap().to_string());
                            match websocket.write_message(message_write) {
                                Ok(_) => {
                                    // Do nothing
                                },
                                Err(error) => {
                                    println!("error client_reader socket: {}", error);
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
                                    websocket.write_message(tungstenite::Message::Text("check_back".to_string())).unwrap();
                                } else {
                                    println!("message not handled: {}", message);
                                }
                            },
                            Err(error) => {
                                println!("error: {}", error);
                                break;
                            }
                        }
                    }
                } else {
                    println!("message not handled: {}", message);
                }
            });
        }
    });
    // Compute thread
    {
        let data_lock_clone = Arc::clone(&data_lock);
        let mut previous_now = SystemTime::now();
        loop {
            let mut data_write = data_lock_clone.write().unwrap();
            pipeline.step(
                &gravity,
                &integration_parameters,
                &mut broad_phase,
                &mut narrow_phase,
                &mut data_write.bodies,
                &mut colliders,
                &mut joints,
                &event_handler
            );
            let now = SystemTime::now();
            let duration_ms = now.duration_since(previous_now).unwrap().as_millis();
            previous_now = now;
            data_write.step += 1;
            // println!("step:  {}ms", duration_ms);
        }
    }
}
