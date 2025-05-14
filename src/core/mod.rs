pub mod ecs;
pub mod physics;
pub mod renderer;
pub mod audio;
pub mod input;

pub use ecs::*;
pub use physics::*;
pub use renderer::*;
pub use audio::*;
pub use input::*;

/// Игровой движок, объединяющий все основные системы
pub struct Engine {
    pub ecs_manager: ecs::EcsManager,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            ecs_manager: ecs::EcsManager::new(),
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.ecs_manager.update(delta_time);
    }
} 