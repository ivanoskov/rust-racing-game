use crate::core::ecs::{System, Resource};
use crate::core::physics::{RigidBodyComponent, ColliderComponent, RigidBodyType, TransformComponent, ColliderShapeType};
use crate::core::input::{InputAction};
use glam::{Vec3, Quat};
use hecs::World;
use std::collections::HashMap;
use rapier3d::prelude::{RigidBodySet, ColliderSet, RigidBodyBuilder, ColliderBuilder};
use rapier3d::math::Vector;
use rapier3d::na::Vector3;

/// Компонент автомобиля
pub struct CarComponent {
    pub name: String,
    pub mass: f32,
    pub max_engine_force: f32,
    pub max_brake_force: f32,
    pub max_steering_angle: f32,
    pub steering_speed: f32,
    pub wheel_base: f32,
    pub engine_position: Vec3,
    pub center_of_mass: Vec3,
    
    // Текущие состояния
    pub current_speed: f32,
    pub current_rpm: f32,
    pub current_gear: i32,
    pub current_steering: f32,
    pub throttle: f32,
    pub brake: f32,
    pub handbrake: f32,
    
    // Характеристики двигателя
    pub torque_curve: Vec<(f32, f32)>, // RPM, torque
    pub gear_ratios: Vec<f32>,
    pub final_drive_ratio: f32,
    pub idle_rpm: f32,
    pub max_rpm: f32,
    pub redline_rpm: f32,
}

impl Default for CarComponent {
    fn default() -> Self {
        Self {
            name: "Default Car".to_string(),
            mass: 1500.0,
            max_engine_force: 10000.0,
            max_brake_force: 15000.0,
            max_steering_angle: 0.5,
            steering_speed: 2.0,
            wheel_base: 2.5,
            engine_position: Vec3::new(0.0, 0.5, 1.5),
            center_of_mass: Vec3::new(0.0, 0.5, 0.0),
            
            current_speed: 0.0,
            current_rpm: 800.0,
            current_gear: 1,
            current_steering: 0.0,
            throttle: 0.0,
            brake: 0.0,
            handbrake: 0.0,
            
            torque_curve: vec![
                (1000.0, 200.0),
                (2000.0, 300.0),
                (3000.0, 350.0),
                (4000.0, 400.0),
                (5000.0, 420.0),
                (6000.0, 380.0),
                (7000.0, 350.0),
                (8000.0, 300.0),
            ],
            gear_ratios: vec![3.5, 2.5, 1.8, 1.3, 1.0, 0.8],
            final_drive_ratio: 3.7,
            idle_rpm: 800.0,
            max_rpm: 8000.0,
            redline_rpm: 7000.0,
        }
    }
}

/// Компонент колеса
pub struct WheelComponent {
    pub radius: f32,
    pub width: f32,
    pub position: Vec3,  // Позиция относительно кузова автомобиля
    pub suspension_rest_length: f32,
    pub suspension_stiffness: f32,
    pub suspension_damping: f32,
    pub suspension_travel: f32,
    pub friction: f32,
    pub steering: bool,  // Управляемое ли колесо
    pub powered: bool,   // Ведущее ли колесо
    
    // Состояние
    pub grounded: bool,
    pub suspension_length: f32,
    pub suspension_force: f32,
    pub wheel_speed: f32,
    pub slip_ratio: f32,
    pub slip_angle: f32,
    pub lateral_force: f32,
    pub longitudinal_force: f32,
}

impl Default for WheelComponent {
    fn default() -> Self {
        Self {
            radius: 0.35,
            width: 0.25,
            position: Vec3::ZERO,
            suspension_rest_length: 0.3,
            suspension_stiffness: 35000.0,
            suspension_damping: 4500.0,
            suspension_travel: 0.15,
            friction: 1.0,
            steering: false,
            powered: false,
            
            grounded: false,
            suspension_length: 0.3,
            suspension_force: 0.0,
            wheel_speed: 0.0,
            slip_ratio: 0.0,
            slip_angle: 0.0,
            lateral_force: 0.0,
            longitudinal_force: 0.0,
        }
    }
}

/// Компонент, связывающий автомобиль с колесами
pub struct CarWheelBindingComponent {
    pub car_entity: hecs::Entity,
    pub wheel_entities: Vec<hecs::Entity>,
}

/// Система управления автомобилем
pub struct CarControlSystem;

impl System for CarControlSystem {
    fn update(&mut self, world: &mut World, delta_time: f32) {
        // Получение состояний ввода 
        let input_states = {
            let input = world
                .query_mut::<&Resource<HashMap<InputAction, f32>>>()
                .into_iter()
                .next()
                .map(|(_, res)| &res.0);
            
            match input {
                Some(states) => states.clone(),
                None => return,
            }
        };
        
        // Обработка ввода для всех автомобилей
        for (_, car) in world.query_mut::<&mut CarComponent>() {
            // Обновляем дроссель
            car.throttle = *input_states.get(&InputAction::Accelerate).unwrap_or(&0.0);
            
            // Обновляем тормоз
            car.brake = *input_states.get(&InputAction::Brake).unwrap_or(&0.0);
            
            // Обновляем ручной тормоз
            car.handbrake = *input_states.get(&InputAction::Handbrake).unwrap_or(&0.0);
            
            // Рулевое управление
            let steer_left = *input_states.get(&InputAction::SteerLeft).unwrap_or(&0.0);
            let steer_right = *input_states.get(&InputAction::SteerRight).unwrap_or(&0.0);
            let target_steering = (steer_right - steer_left) * car.max_steering_angle;
            
            // Плавное изменение угла поворота руля
            if (target_steering - car.current_steering).abs() > 0.01 {
                car.current_steering = car.current_steering + 
                    (target_steering - car.current_steering) * car.steering_speed * delta_time;
            } else {
                car.current_steering = target_steering;
            }
            
            // Переключение передач (здесь можно реализовать автоматическую коробку передач)
            let shift_up = *input_states.get(&InputAction::ShiftUp).unwrap_or(&0.0) > 0.5;
            let shift_down = *input_states.get(&InputAction::ShiftDown).unwrap_or(&0.0) > 0.5;
            
            if shift_up && car.current_gear < car.gear_ratios.len() as i32 - 1 {
                car.current_gear += 1;
            } else if shift_down && car.current_gear > 0 {
                car.current_gear -= 1;
            }
        }
    }
}

/// Система физики автомобиля
pub struct CarPhysicsSystem;

impl System for CarPhysicsSystem {
    fn update(&mut self, world: &mut World, _delta_time: f32) {
        // Сначала соберем все данные, которые нам нужны
        
        // Получаем все связи автомобилей с колесами
        let bindings: Vec<(hecs::Entity, Vec<hecs::Entity>)> = {
            world.query_mut::<&CarWheelBindingComponent>()
                .into_iter()
                .map(|(_, binding)| (binding.car_entity, binding.wheel_entities.clone()))
                .collect()
        };
        
        // Обновляем каждый автомобиль и его колеса по отдельности
        for (car_entity, wheel_entities) in bindings {
            // Обновляем автомобиль
            if let Ok((_car, _car_body)) = world.query_one_mut::<(&mut CarComponent, &RigidBodyComponent)>(car_entity) {
                // Обновляем колеса
                for wheel_entity in wheel_entities {
                    if let Ok(wheel) = world.query_one_mut::<&mut WheelComponent>(wheel_entity) {
                        // Обновляем углы поворота для управляемых колес
                        if wheel.steering {
                            // Применить текущий угол поворота руля
                            // ...
                        }
                        
                        // Обновляем вращение колеса и воздействие на автомобиль
                        if wheel.powered {
                            // Рассчитать крутящий момент двигателя
                            // Применить к колесу
                            // ...
                        }
                        
                        // Обновляем суспензию и контакт с поверхностью
                        // ...
                    }
                }
                
                // Обновляем RPM на основе скорости и передачи
                // ...
                
                // Обновляем скорость из физической скорости тела
                // ...
            }
        }
    }
}

/// Создает полную сущность автомобиля с колесами
pub fn create_car_entity(
    world: &mut World,
    model_name: &str,
    position: Vec3,
    rotation: Quat,
) -> hecs::Entity {
    // Создаем базовый компонент автомобиля
    let car_component = CarComponent {
        name: model_name.to_string(),
        ..Default::default()
    };
    
    // Создаем физический компонент и трансформацию
    let transform = TransformComponent {
        position,
        rotation,
        ..Default::default()
    };
    
    // Получаем ресурс с физическими телами
    let mut resource_query = world.query_mut::<&mut Resource<(RigidBodySet, ColliderSet)>>();
    if let Some((_, resource)) = resource_query.into_iter().next() {
        let (rigid_body_set, collider_set) = &mut resource.0;
        
        // Создаем физическое тело для автомобиля
        let rb = RigidBodyBuilder::dynamic()
            .translation(Vector3::new(position.x, position.y, position.z))
            .build();
        
        // Создаем коллайдер (примерные размеры)
        let collider = ColliderBuilder::cuboid(1.0, 0.5, 2.0)
            .restitution(0.2)
            .friction(0.7)
            .build();
        
        // Добавляем в наборы
        let rb_handle = rigid_body_set.insert(rb);
        let collider_handle = collider_set.insert(collider);
        
        // Создаем компоненты
        let rigid_body = RigidBodyComponent {
            handle: rb_handle,
            body_type: RigidBodyType::Dynamic,
        };
        
        let collider_component = ColliderComponent {
            handle: collider_handle,
            shape_type: ColliderShapeType::Box,
        };
        
        // Создаем сущность автомобиля
        let car_entity = world.spawn((car_component, transform, rigid_body, collider_component));
        
        // Создаем колеса для автомобиля
        let wheel_entities = create_wheels_for_car(world, car_entity);
        
        // Создаем компонент связи между автомобилем и колесами
        let binding = CarWheelBindingComponent {
            car_entity,
            wheel_entities,
        };
        
        world.spawn((binding,));
        
        return car_entity;
    } else {
        // Если ресурс не найден, создаем сущность без физики
        let rigid_body = RigidBodyComponent {
            handle: Default::default(),
            body_type: RigidBodyType::Dynamic,
        };
        
        let collider = ColliderComponent {
            handle: Default::default(),
            shape_type: Default::default(),
        };
        
        // Создаем сущность автомобиля
        let car_entity = world.spawn((car_component, transform, rigid_body, collider));
        
        // Создаем колеса для автомобиля
        let wheel_entities = create_wheels_for_car(world, car_entity);
        
        // Создаем компонент связи между автомобилем и колесами
        let binding = CarWheelBindingComponent {
            car_entity,
            wheel_entities,
        };
        
        world.spawn((binding,));
        
        return car_entity;
    }
}

/// Создает колеса для автомобиля
fn create_wheels_for_car(world: &mut World, _car_entity: hecs::Entity) -> Vec<hecs::Entity> {
    let mut wheel_entities = Vec::new();
    
    // Получаем характеристики автомобиля для расположения колес
    let wheel_base = 2.5;  // В реальном приложении берется из компонента автомобиля
    let track_width = 1.8;
    
    // Создаем 4 колеса: переднее левое, переднее правое, заднее левое, заднее правое
    let wheel_positions = [
        Vec3::new(-track_width/2.0, 0.0, wheel_base/2.0),
        Vec3::new(track_width/2.0, 0.0, wheel_base/2.0),
        Vec3::new(-track_width/2.0, 0.0, -wheel_base/2.0),
        Vec3::new(track_width/2.0, 0.0, -wheel_base/2.0),
    ];
    
    for (i, position) in wheel_positions.iter().enumerate() {
        let is_front = i < 2;
        
        let wheel = WheelComponent {
            position: *position,
            steering: is_front,
            powered: !is_front,  // Задний привод
            ..Default::default()
        };
        
        let transform = TransformComponent {
            position: *position,
            ..Default::default()
        };
        
        // Заглушки для физических компонентов
        let rigid_body = RigidBodyComponent {
            handle: Default::default(),
            body_type: RigidBodyType::Dynamic,
        };
        
        let collider = ColliderComponent {
            handle: Default::default(),
            shape_type: Default::default(),
        };
        
        let wheel_entity = world.spawn((wheel, transform, rigid_body, collider));
        wheel_entities.push(wheel_entity);
    }
    
    wheel_entities
} 