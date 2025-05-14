use crate::core::ecs::{System, Resource};
use hecs::World;
use rapier3d::prelude::*;
use glam::{Vec3, Quat};

/// Компонент физического тела
pub struct RigidBodyComponent {
    pub handle: RigidBodyHandle,
    pub body_type: RigidBodyType,
}

/// Компонент коллайдера
pub struct ColliderComponent {
    pub handle: ColliderHandle,
    pub shape_type: ColliderShapeType,
}

/// Типы коллайдеров
#[derive(Debug, Clone, Copy)]
pub enum ColliderShapeType {
    Box,
    Ball,
    Capsule,
    Convex,
    Heightfield,
    Trimesh,
    Compound,
    Custom,
}

impl Default for ColliderShapeType {
    fn default() -> Self {
        ColliderShapeType::Box
    }
}

/// Физическая система
pub struct PhysicsSystem {
    gravity: Vector<Real>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
}

impl PhysicsSystem {
    pub fn new() -> Self {
        Self {
            gravity: vector![0.0, -9.81, 0.0],
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
        }
    }

    pub fn set_gravity(&mut self, gravity: Vec3) {
        self.gravity = vector![gravity.x, gravity.y, gravity.z];
    }

    pub fn create_rigid_body(&self, position: Vec3, _rotation: Quat, body_type: RigidBodyType) -> RigidBody {
        let rb = match body_type {
            RigidBodyType::Dynamic => RigidBodyBuilder::dynamic()
                .translation(vector![position.x, position.y, position.z])
                .build(),
            RigidBodyType::Static => RigidBodyBuilder::fixed()
                .translation(vector![position.x, position.y, position.z])
                .build(),
            RigidBodyType::Kinematic => RigidBodyBuilder::kinematic_position_based()
                .translation(vector![position.x, position.y, position.z])
                .build(),
        };

        rb
    }

    pub fn create_box_collider(&self, half_extents: Vec3, restitution: f32, friction: f32) -> Collider {
        ColliderBuilder::cuboid(half_extents.x, half_extents.y, half_extents.z)
            .restitution(restitution)
            .friction(friction)
            .build()
    }

    // Публичный метод для обновления физики, который можно вызывать напрямую
    pub fn process(&mut self, world: &mut World, delta_time: f32) {
        // Содержимое такое же, как в функции update
        // Обновляем трансформации после физического шага
        // Собираем данные о положении физических тел и компонентах
        let body_handles: Vec<(hecs::Entity, RigidBodyHandle)> = world
            .query::<&RigidBodyComponent>()
            .iter()
            .map(|(entity, rb)| (entity, rb.handle))
            .collect();
            
        let mut updates = Vec::new();
        
        {
            // Получаем resource с физическими телами
            let resource = &world.query_mut::<&Resource<(RigidBodySet, ColliderSet)>>()
                .into_iter().next().unwrap().1.0;
            let (rigid_body_set, _) = resource;
            
            // Обрабатываем собранные ранее данные без повторного заимствования world
            for (entity, handle) in body_handles {
                if let Some(rb) = rigid_body_set.get(handle) {
                    let pos = rb.translation();
                    let rot = rb.rotation();
                    
                    updates.push((
                        entity,
                        Vec3::new(pos.x, pos.y, pos.z),
                        Quat::from_xyzw(rot.i, rot.j, rot.k, rot.w)
                    ));
                }
            }
        }
        
        // Второй блок - обновляем компоненты трансформации
        for (entity, position, rotation) in updates {
            // Для каждой сущности делаем отдельный запрос query_one_mut
            if let Ok(transform) = world.query_one_mut::<&mut TransformComponent>(entity) {
                transform.position = position;
                transform.rotation = rotation;
            }
        }
    }
}

// Используем отдельную реализацию System только для совместимости со старым кодом
impl System for PhysicsSystem {
    fn update(&mut self, world: &mut World, delta_time: f32) {
        self.process(world, delta_time);
    }
}

/// Компонент трансформации
pub struct TransformComponent {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

/// Типы физических тел
#[derive(Debug, Clone, Copy)]
pub enum RigidBodyType {
    Dynamic,
    Static,
    Kinematic,
}

impl Default for RigidBodyType {
    fn default() -> Self {
        RigidBodyType::Dynamic
    }
}

/// Пример данных для обработки столкновений
pub struct CollisionEvent {
    pub entity1: hecs::Entity,
    pub entity2: hecs::Entity,
    pub point: Vec3,
    pub normal: Vec3,
    pub impulse: f32,
} 