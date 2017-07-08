extern crate amethyst;

extern crate ncollide;
extern crate nalgebra;

// use nalgebra::Vector2;
// use ncollide::shape::Cuboid;
// use ncollide::Point2;
use nalgebra::Point2;
use ncollide::shape::ConvexHull;

use amethyst::{Application, Event, State, Trans, VirtualKeyCode, WindowEvent};
use amethyst::asset_manager::AssetManager;
use amethyst::gfx_device::DisplayConfig;
use amethyst::ecs::{Component, VecStorage, Fetch, FetchMut, Join, System, WriteStorage, World};
use amethyst::ecs::components::{Mesh, LocalTransform, Texture, Transform};
use amethyst::ecs::resources::{Camera, InputHandler, Projection, Time};
use amethyst::ecs::systems::TransformSystem;
use amethyst::renderer::{Pipeline, VertexPosNormal};

struct Qix;

impl State for Qix {
    fn on_start(&mut self, world: &mut World, assets: &mut AssetManager, pipe: &mut Pipeline) {
        use amethyst::ecs::resources::{Camera, InputHandler, Projection, ScreenDimensions};
        use amethyst::renderer::Layer;
        use amethyst::renderer::pass::{Clear, DrawFlat};

        let layer = Layer::new("main",
                               vec![Clear::new([0.0, 0.0, 0.0, 1.0]),
                               DrawFlat::new("main", "main")]);

        pipe.layers.push(layer);

        {
            // TODO: handle new dimensions if screen size changes!
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
            .with(Position{x: 0.5, y: 1.0})
            .with(Velocity{x: 0.0, y: 0.0})
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

#[derive(Debug)]
struct Velocity {
    pub x: f32,
    pub y: f32
}

enum Direction {
    Up,
    Right,
    Down,
    Left,
    None
}

impl Velocity {
    fn direction(&self) -> Direction {
        if self.x > 0. {
            return Direction::Right;
        } else if self.x < 0. {
            return Direction::Left;
        } else if self.y > 0. {
            return Direction::Down;
        } else if self.y < 0. {
            return Direction::Up;
        } else {
            return Direction::None;
        }
    }
}

impl Component for Velocity {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
struct Position {
    pub x: f32,
    pub y: f32
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}

struct Marker {
    pub size: f32,
    pub is_stix: bool,
    pub is_fast: bool
}

impl Marker {
    fn new() -> Marker {
        Marker {
            size: 1.0,
            is_stix: false,
            is_fast: false
        }
    }
}

impl Component for Marker {
    type Storage = VecStorage<Marker>;
}

struct UpdatePos;
impl<'a> System<'a> for UpdatePos {
    type SystemData = (WriteStorage<'a, Marker>, 
                       WriteStorage<'a, Position>,
                       WriteStorage<'a, Velocity>,
                       Fetch<'a, Time>);
    fn run(&mut self, (mut markers, mut positions, mut vel, time): Self::SystemData) {
        let delta_time = time.delta_time.subsec_nanos() as f32 / 1.0e9;
        for (marker, pos, vel) in (&mut markers, &mut positions, &mut vel).join() {
            pos.x += vel.x * delta_time;
            pos.y += vel.y * delta_time;
        }
    }
}

struct CollisionSystem;

impl<'a> System<'a> for CollisionSystem {
    type SystemData = (WriteStorage<'a, Marker>, 
                       WriteStorage<'a, Position>,
                       WriteStorage<'a, Velocity>);
    fn run(&mut self, (mut markers, mut positions, mut velocities): Self::SystemData) {
        for (marker, pos, vel) in (&mut markers, &mut positions, &mut velocities).join() {
            let mut min_pos = Position{x: 0., y: 0.};
            let mut max_pos = Position{x: 1., y: 1.};

            let is_on_border = pos.x == min_pos.x 
                || pos.x == max_pos.x 
                || pos.y == min_pos.y 
                || pos.y == max_pos.y;

            if !is_on_border {
                match vel.direction() {
                    Direction::Up => {
                        if pos.y < max_pos.y && pos.y > min_pos.y && !marker.is_stix  {
                            min_pos.y = max_pos.y
                        }
                    }
                    Direction::Right => {
                        if pos.x > min_pos.x && pos.x < max_pos.x && !marker.is_stix  {
                            max_pos.x = min_pos.x
                        }
                    }
                    Direction::Down => {
                        if pos.y > min_pos.y && pos.y < max_pos.y && !marker.is_stix  {
                            max_pos.y = min_pos.y
                        }
                    }
                    Direction::Left => {
                        if pos.x < max_pos.x && pos.x > min_pos.x && !marker.is_stix  {
                            min_pos.x = max_pos.x
                        }
                    }
                    Direction::None => {
                    }
                }
            }

            // todo: search through areas

            // Correct positions if they went out of bounds
            // It should only be possible to go out of one coordinate parameter at a time.
            if pos.x < min_pos.x {
                pos.x = min_pos.x;
            } else if pos.x > max_pos.x {
                pos.x = max_pos.x;
            } else if pos.y < min_pos.y {
                pos.y = min_pos.y;
            } else if pos.y > max_pos.y {
                pos.y = max_pos.y;
            }

            // let points = vec![
            //     Point2::new(-1.0f32, 1.0), Point2::new(-0.5, -0.5),
            //     Point2::new(0.0, 0.5),     Point2::new(0.5, -0.5),
            //     Point2::new(1.0, 1.0)
            // ];
            //
            // let convex = ConvexHull::new(points);
        }

        // // use amethyst::ecs::resources::{InputHandler, Time};
    }
}

struct QixSystem;

unsafe impl Sync for QixSystem {}

impl<'a> System<'a> for QixSystem {
    type SystemData = (
        WriteStorage<'a, Marker>, 
        WriteStorage<'a, Velocity>,
        WriteStorage<'a, LocalTransform>,
        FetchMut<'a, Camera>, 
        FetchMut<'a, InputHandler>);
    fn run(&mut self, (mut markers, mut vel, mut locals, mut camera, 
                       // time,
                       mut input): Self::SystemData) {
        use amethyst::ecs::resources::{Camera, InputHandler, Projection, Time};
        // mut score

        // // Get left and right boundaries of the screen
        // let (left_bound, right_bound, top_bound, bottom_bound) = match camera.proj {
        //     Projection::Orthographic { left, right, top, bottom, .. } => (left, right, top, bottom),
        //     _ => (1.0, 1.0, 1.0, 1.0),
        // };
        //
        //
        let MAX_NORMAL_VELOCITY = 0.5;
        let FAST_VELOCITY_MULTIPLIER = 1.5;

        for (marker, vel) in (&mut markers, &mut vel).join() {
            if input.key_is_pressed(VirtualKeyCode::Up) {
                vel.x = 0.;
                vel.y = -MAX_NORMAL_VELOCITY;
            } else if input.key_is_pressed(VirtualKeyCode::Right) {
                vel.x = MAX_NORMAL_VELOCITY;
                vel.y = 0.;
            } else if input.key_is_pressed(VirtualKeyCode::Down) {
                vel.x = 0.;
                vel.y = MAX_NORMAL_VELOCITY;
            } else if input.key_is_pressed(VirtualKeyCode::Left)  {
                vel.x = -MAX_NORMAL_VELOCITY;
                vel.y = 0.;
            } else {
                vel.x = 0.;
                vel.y = 0.;
            }

            if input.key_is_pressed(VirtualKeyCode::Z) {
                marker.is_stix = true;
            }

            if input.key_is_pressed(VirtualKeyCode::X) {
                marker.is_stix = true;
                marker.is_fast = true;
            }
        }
    }
}

struct RenderSystem;

impl<'a> System<'a> for RenderSystem {
    type SystemData = (WriteStorage<'a, Marker>,
                       WriteStorage<'a, Position>,
                       WriteStorage<'a, LocalTransform>);
    fn run(&mut self, (mut markers, mut positions, mut locals): Self::SystemData) {
        for (marker, position, local) in (&mut markers, &mut positions, &mut locals).join() {
            local.translation[0] = position.x;
            local.translation[1] = position.y;

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
        .register::<Position>()
        .register::<Velocity>()
        .register::<Marker>()
        .with::<QixSystem>(QixSystem, "qix_system", &[])
        .with::<UpdatePos>(UpdatePos, "update_position_system", &[])
        .with::<CollisionSystem>(CollisionSystem, "collision_system", &[])
        .with::<RenderSystem>(RenderSystem, "render_system", &[])
        .with::<TransformSystem>(TransformSystem::new(), "transform_system", &["qix_system"])
        .done();
    game.run();
}

