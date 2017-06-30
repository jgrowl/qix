extern crate amethyst;

use amethyst::{Application, Event, State, Trans, VirtualKeyCode, WindowEvent};
use amethyst::asset_manager::AssetManager;
use amethyst::gfx_device::DisplayConfig;
use amethyst::ecs::{World, Join, VecStorage, Component, RunArg, System};
use amethyst::ecs::components::{Mesh, LocalTransform, Texture, Transform};
use amethyst::renderer::{Pipeline, VertexPosNormal};


struct Qix;

impl State for Qix {
    fn on_start(&mut self, world: &mut World, assets: &mut AssetManager, pipe: &mut Pipeline) {
        // println!("Game started!");
        use amethyst::ecs::Gate;

        use amethyst::ecs::resources::{Camera, InputHandler, Projection, ScreenDimensions};
        use amethyst::renderer::Layer;
        use amethyst::renderer::pass::{Clear, DrawFlat};

        let layer = Layer::new("main",
                               vec![Clear::new([0.0, 0.0, 0.0, 1.0]),
                               DrawFlat::new("main", "main")]);

        pipe.layers.push(layer);

        {
            let dim = world.read_resource::<ScreenDimensions>().pass();
            let mut camera = world.write_resource::<Camera>().pass();
            let aspect_ratio = dim.aspect_ratio;
            let eye = [0., 0., 0.1];
            let target = [0., 0., 0.];
            let up = [0., 1., 0.];

            // Get an Orthographic projection
            let proj = Projection::Orthographic {
                left: -1.0 * aspect_ratio,
                right: 1.0 * aspect_ratio,
                bottom: -1.0,
                top: 1.0,
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
        // ball.velocity = [0.5, 0.5];
        world.create_now()
            .with(square.clone())
            .with(marker)
            .with(LocalTransform::default())
            .with(Transform::default())
            .build();
    }

    fn handle_events(&mut self,
                     events: &[WindowEvent],
                     _: &mut World,
                     _: &mut AssetManager,
                     _: &mut Pipeline)
        -> Trans {
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
    pub velocity: i32
}

impl Marker {
    fn new() -> Marker {
        Marker {
            size: 1.0,
            position: [0.,0.],
            velocity: 5
        }
    }
}

impl Component for Marker {
    type Storage = VecStorage<Marker>;
}

struct QixSystem;

unsafe impl Sync for QixSystem {}

impl System<()> for QixSystem {
    fn run(&mut self, arg: RunArg, _: ()) {

        use amethyst::ecs::Gate;
        use amethyst::ecs::resources::{Camera, InputHandler, Projection, Time};


        // Get all needed component storages and resources
        // , planks, locals, time, input, mut score
        let (mut markers, locals, camera) = arg.fetch(|w| {
            (w.write::<Marker>(),
            // w.write::<Plank>(),
            w.write::<LocalTransform>(),
            w.read_resource::<Camera>(),
            // w.read_resource::<Time>(),
            // w.read_resource::<InputHandler>(),
            // w.write_resource::<Score>()
            )
        });

        // Get left and right boundaries of the screen
        let (left_bound, right_bound, top_bound, bottom_bound) = match camera.proj {
            Projection::Orthographic { left, right, top, bottom, .. } => (left, right, top, bottom),
            _ => (1.0, 1.0, 1.0, 1.0),
        };

        let mut locals = locals.pass();

        // Process the marker
        for (marker, local) in (&mut markers, &mut locals).join() {
            marker.position[0] = 0.;
            marker.position[1] = 0.;
                
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
    // let marker = Marker::new();
    //println!("{}", marker.position);

    let config = DisplayConfig::default();
    let mut game = Application::build(Qix, config)
        .register::<Marker>()
        .with::<QixSystem>(QixSystem, "QixSystem", 1)
        .done();
    game.run();
}

