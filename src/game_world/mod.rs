pub mod car;
pub mod track;
pub mod environment;

use crate::core::ecs::{EcsManager, Resource};
use crate::core::physics::PhysicsSystem;
use rapier3d::prelude::{RigidBodySet, ColliderSet};

/// Менеджер игрового мира
pub struct GameWorldManager {
    pub physics_system: PhysicsSystem,
}

impl GameWorldManager {
    pub fn new() -> Self {
        Self {
            physics_system: PhysicsSystem::new(),
        }
    }
    
    /// Инициализация физического мира и создание необходимых ресурсов
    pub fn initialize_physics(&self, ecs_manager: &mut EcsManager) {
        // Создаем наборы физических тел и коллайдеров
        let rigid_body_set = RigidBodySet::new();
        let collider_set = ColliderSet::new();
        
        // Добавляем их как ресурс в ECS мир
        ecs_manager.world.spawn((Resource((rigid_body_set, collider_set)),));
    }
    
    /// Регистрация всех необходимых систем в ECS-менеджере
    pub fn register_systems(&self, ecs_manager: &mut EcsManager) {
        // Физическую систему не регистрируем, будем вызывать напрямую
        
        // Регистрация систем для автомобилей
        ecs_manager.register_system(car::CarControlSystem);
        ecs_manager.register_system(car::CarPhysicsSystem);
        
        // Здесь будут регистрироваться другие системы для трасс и окружения
    }
} 