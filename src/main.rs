extern crate amethyst;

use amethyst::{Application, Event, State, Trans, VirtualKeyCode, WindowEvent};
use amethyst::asset_manager::AssetManager;
use amethyst::gfx_device::DisplayConfig;
// use amethyst::ecs::{World, Join, VecStorage, Component, RunArg, System};
use amethyst::ecs::{Component, VecStorage, Fetch, FetchMut, Join, System, WriteStorage, World};

use amethyst::ecs::components::{Mesh, LocalTransform, Texture, Transform};
use amethyst::ecs::resources::{Camera, InputHandler, Projection, Time};
use amethyst::ecs::systems::TransformSystem;
use amethyst::renderer::{Pipeline, VertexPosNormal};


struct Qix;

impl State for Qix {
    fn on_start(&mut self, world: &mut World, assets: &mut AssetManager, pipe: &mut Pipeline) {
        // use amethyst::ecs::Gate;

        use amethyst::ecs::resources::{Camera, InputHandler, Projection, ScreenDimensions};
        use amethyst::renderer::Layer;
        use amethyst::renderer::pass::{Clear, DrawFlat};

        let layer = Layer::new("main",
                               vec![Clear::new([0.0, 0.0, 0.0, 1.0]),
                               DrawFlat::new("main", "main")]);

        pipe.layers.push(layer);

        {
            let dim = world.read_resource::<ScreenDimensions>();
            let mut camera = world.write_resource::<Camera>();
            let aspect_ratio = dim.aspect_ratio;
            let eye = [0., 0., 0.1];
            let target = [0., 0., 0.];
            let up = [0., 1., 0.];

            // Get an Orthographic projection
            let proj = Projection::Orthographic {
                left: 0.0,
                top: 0.0,
                right: 1.0,
                bottom: 1.0,
                near: 0.0,
                far: 1.0,
            };

            camera.proj = proj;
            camera.eye = eye;
            camera.target = target;
            camera.up = up;
        }

        world.add_resource::<InputHandler>(InputHandler::new());

        // Generate a square mesh
        assets.register_asset::<Mesh>();
        assets.register_asset::<Texture>();
        assets.load_asset_from_data::<Texture, [f32; 4]>("white", [1.0, 1.0, 1.0, 1.0]);
        let square_verts = gen_rectangle(1.0, 1.0);
        assets.load_asset_from_data::<Mesh, Vec<VertexPosNormal>>("square", square_verts);
        let square = assets.create_renderable("square", "white", "white", "white", 1.0).unwrap();


        // Create a marker entity
        let mut marker = Marker::new();
        marker.size = 0.02;
        world.create_entity()
            .with(square.clone())
            .with(marker)
            .with(LocalTransform::default())
            .with(Transform::default())
            .build();
    }

    fn handle_events(&mut self,
                     events: &[WindowEvent],
                     world: &mut World,
                     _: &mut AssetManager,
                     _: &mut Pipeline)
        -> Trans {
            // use amethyst::ecs::Gate;
            use amethyst::ecs::resources::InputHandler;

            let mut input = world.write_resource::<InputHandler>();
            input.update(events);

            for e in events {
                match **e {
                    Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => return Trans::Quit,
                    Event::Closed => return Trans::Quit,
                    _ => (),
                }
            }
            Trans::None
        }

    fn update(&mut self, _: &mut World, _: &mut AssetManager, _: &mut Pipeline) -> Trans {
        // Trans::Quit
        Trans::None
    }

    fn on_stop(&mut self, _: &mut World, _: &mut AssetManager, _: &mut Pipeline) {
        println!("Game stopped!");
    }
}

struct Marker {
    pub size: f32,
    pub position: [f32; 2],
    pub velocity: f32
}

impl Marker {
    fn new() -> Marker {
        Marker {
            size: 1.0,
            position: [0.5,1.0],
            velocity: 0.25
        }
    }
}

impl Component for Marker {
    type Storage = VecStorage<Marker>;
}

struct QixSystem;

unsafe impl Sync for QixSystem {}

impl<'a> System<'a> for QixSystem {
    type SystemData = (
        WriteStorage<'a, Marker>, 
        WriteStorage<'a, LocalTransform>,
        FetchMut<'a, Camera>, 
        Fetch<'a, Time>,
        FetchMut<'a, InputHandler>);
    fn run(&mut self, (mut markers, mut locals, mut camera, time, mut input): Self::SystemData) {
//         }
// }
//
//
// impl System<()> for QixSystem {
//     fn run(&mut self, arg: RunArg, _: ()) {

        // use amethyst::ecs::Gate;
        use amethyst::ecs::resources::{Camera, InputHandler, Projection, Time};
        // mut score
        // let (mut markers, locals, camera, time, input) = arg.fetch(|w| {
        //     (w.write::<Marker>(),
        //     w.write::<LocalTransform>(),
        //     w.read_resource::<Camera>(),
        //     w.read_resource::<Time>(),
        //     w.read_resource::<InputHandler>())
        //         // ,w.write_resource::<Score>()
        // });

        // Get left and right boundaries of the screen
        let (left_bound, right_bound, top_bound, bottom_bound) = match camera.proj {
            Projection::Orthographic { left, right, top, bottom, .. } => (left, right, top, bottom),
            _ => (1.0, 1.0, 1.0, 1.0),
        };

        let delta_time = time.delta_time.subsec_nanos() as f32 / 1.0e9;

        // let mut locals = locals;

        #[derive(PartialEq, Eq)]
        enum Side {
            Top,
            Right,
            Bottom,
            Left,
        };

        // Process the marker
        for (marker, local) in (&mut markers, &mut locals).join() {

            let mut side: Side = Side::Bottom;

            if marker.position[0] == 0. {
                side = Side::Left;
            } else if marker.position[1] == 0. {
                side = Side::Top;
            } else if marker.position[0] == 1. {
                side = Side::Right;
            } else if marker.position[1] == 1. {
                side = Side::Bottom
            }


            if side == Side::Bottom {
                if input.key_is_pressed(VirtualKeyCode::Right) {
                    let position = marker.position[0] + marker.velocity * delta_time;
                    marker.position[0] = if position >= 1. { 1. } else {position}
                    // if new_position >= 1. {
                    //     marker.position[0] = 1.;
                    // } else {
                    //     marker.position[0] = new_position;
                    // }
                }

                if input.key_is_pressed(VirtualKeyCode::Left) {
                    let position = marker.position[0] - marker.velocity * delta_time;
                    marker.position[0] = if position <= 0. { 0.} else {position}
                    // if new_position <= 0. {
                    //     marker.position[0] = 0.;
                    // } else {
                    //     marker.position[0] = new_position;
                    // }
                }
            } else if side == Side::Left {
                if input.key_is_pressed(VirtualKeyCode::Up) {
                    let position = marker.position[1] - marker.velocity * delta_time;
                    marker.position[1] = if position <= 0. {0.} else {position}
                    // if position <= 0. {
                    //     marker.position[1] = 0.;
                    // } else {
                    //     marker.position[1] = position;
                    // }
                }

                if input.key_is_pressed(VirtualKeyCode::Down) {
                    let position = marker.position[1] + marker.velocity * delta_time;
                    marker.position[1] = if position >= 1. { 1. } else {position}
                    // if position >= 1. {
                    //     marker.position[1] = 1.;
                    // } else {
                    //     marker.position[1] = position;
                    // }
                }
            } else if side == Side::Top {
                if input.key_is_pressed(VirtualKeyCode::Left) {
                    let position = marker.position[0] - marker.velocity * delta_time;
                    marker.position[0] = if position <= 0. { 0. } else { position }
                    // if position <= 0. {
                    //     marker.position[0] = 0.;
                    // } else {
                    //     marker.position[0] = position;
                    // }
                }

                if input.key_is_pressed(VirtualKeyCode::Right) {
                    let position = marker.position[0] + marker.velocity * delta_time;
                    marker.position[0] = if position >= 1. { 1. } else { position }
                    // if position >= 1. {
                    //     marker.position[0] = 1.;
                    // } else {
                    //     marker.position[0] = position;
                    // }
                }
            } else if side == Side::Right {
                if input.key_is_pressed(VirtualKeyCode::Down) {
                    let position = marker.position[1] + marker.velocity * delta_time;
                    marker.position[1] = if position >= 1. { 1. } else { position }
                    // if position >= 1. {
                    //     marker.position[1] = 1.;
                    // } else {
                    //     marker.position[1] = position;
                    // }
                }

                if input.key_is_pressed(VirtualKeyCode::Up) {
                    let position = marker.position[1] - marker.velocity * delta_time;
                    marker.position[1] = if position <=0. {0.} else { position }
                    // if position <= 0. {
                    //     marker.position[1] = 0.;
                    // } else {
                    //     marker.position[1] = position;
                    // }
                }
            }

            local.translation[0] = marker.position[0];
            local.translation[1] = marker.position[1];

            local.scale[0] = marker.size;
            local.scale[1] = marker.size;
        }
    }
}

fn gen_rectangle(w: f32, h: f32) -> Vec<VertexPosNormal> {
    let data: Vec<VertexPosNormal> = vec![VertexPosNormal {
        pos: [-w / 2., -h / 2., 0.],
        normal: [0., 0., 1.],
        tex_coord: [0., 0.],
    },
    VertexPosNormal {
        pos: [w / 2., -h / 2., 0.],
        normal: [0., 0., 1.],
        tex_coord: [1., 0.],
    },
    VertexPosNormal {
        pos: [w / 2., h / 2., 0.],
        normal: [0., 0., 1.],
        tex_coord: [1., 1.],
    },
    VertexPosNormal {
        pos: [w / 2., h / 2., 0.],
        normal: [0., 0., 1.],
        tex_coord: [1., 1.],
    },
    VertexPosNormal {
        pos: [-w / 2., h / 2., 0.],
        normal: [0., 0., 1.],
        tex_coord: [1., 1.],
    },
    VertexPosNormal {
        pos: [-w / 2., -h / 2., 0.],
        normal: [0., 0., 1.],
        tex_coord: [1., 1.],
    }];
    data
}

fn main() {
    let config = DisplayConfig::default();
    let mut game = Application::build(Qix, config)
        .register::<Marker>()
        // .with::<QixSystem>(QixSystem, "QixSystem", 1)
        .with::<QixSystem>(QixSystem, "pong_system", &[])
        .with::<TransformSystem>(TransformSystem::new(), "transform_system", &["pong_system"])
        .done();
    game.run();
}

