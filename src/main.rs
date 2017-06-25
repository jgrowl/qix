extern crate amethyst;

use amethyst::{Application, State, Trans};
use amethyst::asset_manager::AssetManager;
use amethyst::ecs::World;
use amethyst::gfx_device::DisplayConfig;
use amethyst::renderer::Pipeline;

struct HelloWorld;

impl State for HelloWorld {
    fn on_start(&mut self, _: &mut World, _: &mut AssetManager, _: &mut Pipeline) {
        println!("Game started!");
    }

    fn update(&mut self, _: &mut World, _: &mut AssetManager, _: &mut Pipeline) -> Trans {
        println!("Hello from Amethyst!");
        Trans::Quit
    }

    fn on_stop(&mut self, _: &mut World, _: &mut AssetManager, _: &mut Pipeline) {
        println!("Game stopped!");
    }
}

fn main() {
    let config = DisplayConfig::default();
    let mut game = Application::build(HelloWorld, config).done();
    game.run();
}
