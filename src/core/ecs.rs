use hecs::{Entity, World};
use std::any::TypeId;
use std::collections::HashMap;

/// Основной класс ECS, который управляет всеми сущностями и системами
pub struct EcsManager {
    pub world: World,
    systems: HashMap<TypeId, Box<dyn System>>,
    system_execution_order: Vec<TypeId>,
}

impl EcsManager {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            systems: HashMap::new(),
            system_execution_order: Vec::new(),
        }
    }

    pub fn create_entity(&mut self, components: impl hecs::DynamicBundle) -> Entity {
        self.world.spawn(components)
    }

    pub fn register_system<S: System + 'static>(&mut self, system: S) {
        let type_id = TypeId::of::<S>();
        self.systems.insert(type_id, Box::new(system));
        self.system_execution_order.push(type_id);
    }

    pub fn set_system_execution_order(&mut self, order: Vec<TypeId>) {
        self.system_execution_order = order;
    }

    pub fn update(&mut self, delta_time: f32) {
        for system_type_id in &self.system_execution_order {
            if let Some(system) = self.systems.get_mut(system_type_id) {
                system.update(&mut self.world, delta_time);
            }
        }
    }
}

/// Трейт для систем в ECS
pub trait System: Send + Sync {
    fn update(&mut self, world: &mut World, delta_time: f32);
}

/// Компонент ресурсов, который не привязан к конкретной сущности
pub struct Resource<T>(pub T);

/// Обертка для событий в ECS
pub struct EventQueue<T> {
    events: Vec<T>,
}

impl<T> EventQueue<T> {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn publish(&mut self, event: T) {
        self.events.push(event);
    }

    pub fn consume<F: FnMut(&T)>(&mut self, mut f: F) {
        for event in &self.events {
            f(event);
        }
        self.clear();
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}

/// Менеджер ресурсов для ECS
pub struct ResourceManager {
    resources: HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn insert<T: 'static + Send + Sync>(&mut self, resource: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(resource));
    }

    pub fn get<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|res| res.downcast_ref::<T>())
    }

    pub fn get_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        self.resources
            .get_mut(&TypeId::of::<T>())
            .and_then(|res| res.downcast_mut::<T>())
    }
} 