use crate::core::ecs::{System};
use crate::core::physics::{TransformComponent};
use glam::{Vec3, Quat};
use hecs::World;

/// Компонент погоды
pub struct WeatherComponent {
    pub weather_type: WeatherType,
    pub intensity: f32,
    pub transition_time: f32,
    pub current_time: f32,
    pub target_weather: Option<WeatherType>,
}

/// Типы погоды
#[derive(Clone)]
pub enum WeatherType {
    Clear,
    Cloudy,
    Rain,
    Storm,
    Fog,
    Snow,
}

impl Default for WeatherType {
    fn default() -> Self {
        WeatherType::Clear
    }
}

/// Компонент времени суток
pub struct TimeOfDayComponent {
    pub hour: f32,           // 0-24
    pub minute: f32,         // 0-60
    pub day_length: f32,     // В секундах реального времени
    pub time_scale: f32,     // Множитель скорости течения времени
    pub sun_position: Vec3,  // Текущая позиция солнца
    pub moon_position: Vec3, // Текущая позиция луны
}

impl Default for TimeOfDayComponent {
    fn default() -> Self {
        Self {
            hour: 12.0,
            minute: 0.0,
            day_length: 1200.0,  // 20 минут реального времени на день
            time_scale: 1.0,
            sun_position: Vec3::new(0.0, 1.0, 0.0),
            moon_position: Vec3::new(0.0, -1.0, 0.0),
        }
    }
}

/// Компонент разрушаемого объекта
pub struct DestructibleComponent {
    pub health: f32,
    pub max_health: f32,
    pub destroyed: bool,
    pub destruction_threshold: f32,
    pub destruction_stages: Vec<DestructionStage>,
    pub current_stage: usize,
}

/// Стадия разрушения объекта
pub struct DestructionStage {
    pub health_threshold: f32,
    pub mesh_id: usize,
    pub effects: Vec<String>,
    pub sounds: Vec<String>,
}

/// Компонент для окружающих объектов
pub struct EnvironmentObjectComponent {
    pub object_type: EnvironmentObjectType,
    pub can_collide: bool,
    pub is_static: bool,
}

/// Типы окружающих объектов
pub enum EnvironmentObjectType {
    Tree,
    Rock,
    Building,
    Fence,
    TrafficLight,
    StreetLight,
    Sign,
    Barrier,
    Decoration,
    Custom(String),
}

/// Система управления погодой
pub struct WeatherSystem;

impl System for WeatherSystem {
    fn update(&mut self, world: &mut World, delta_time: f32) {
        // Обновляем погоду
        for (_, weather) in world.query_mut::<&mut WeatherComponent>() {
            // Обрабатываем переход между типами погоды, если нужно
            if let Some(target) = &weather.target_weather {
                weather.current_time += delta_time;
                
                if weather.current_time >= weather.transition_time {
                    // Завершен переход
                    weather.weather_type = (*target).clone();
                    weather.target_weather = None;
                    weather.current_time = 0.0;
                } else {
                    // Продолжается переход, обновляем интенсивность
                    let _progress = weather.current_time / weather.transition_time;
                    // Здесь можно плавно менять визуальные эффекты погоды
                }
            }
            
            // Другие эффекты погоды...
        }
    }
}

/// Система управления временем суток
pub struct TimeOfDaySystem;

impl System for TimeOfDaySystem {
    fn update(&mut self, world: &mut World, delta_time: f32) {
        // Обновляем время суток
        for (_, time) in world.query_mut::<&mut TimeOfDayComponent>() {
            // Вычисление новых часов и минут
            let real_delta = delta_time * time.time_scale;
            let day_progress = real_delta / time.day_length;
            let time_increment = day_progress * 24.0 * 60.0;  // В минутах
            
            time.minute += time_increment;
            
            // Обрабатываем переходы между часами
            while time.minute >= 60.0 {
                time.minute -= 60.0;
                time.hour += 1.0;
                
                if time.hour >= 24.0 {
                    time.hour -= 24.0;
                }
            }
            
            // Вычисление позиции солнца и луны на основе времени суток
            let day_angle = (time.hour / 24.0 + time.minute / (24.0 * 60.0)) * 2.0 * std::f32::consts::PI;
            
            // Солнце
            time.sun_position = Vec3::new(
                day_angle.sin(),
                day_angle.cos(),
                0.0,
            ).normalize();
            
            // Луна (противоположная сторона от солнца)
            time.moon_position = Vec3::new(
                -day_angle.sin(),
                -day_angle.cos(),
                0.0,
            ).normalize();
        }
    }
}

/// Система управления разрушаемыми объектами
pub struct DestructibleSystem;

impl System for DestructibleSystem {
    fn update(&mut self, world: &mut World, _delta_time: f32) {
        // Обновляем состояние разрушаемых объектов
        for (_, (destructible, _transform)) in world.query_mut::<(&mut DestructibleComponent, &mut TransformComponent)>() {
            if destructible.destroyed {
                continue;
            }
            
            // Проверяем, нужно ли обновить стадию разрушения
            if destructible.health <= destructible.destruction_threshold && !destructible.destroyed {
                destructible.destroyed = true;
                
                // Визуальные и звуковые эффекты разрушения
                // ...
                
                // Изменения физики (например, отключение коллизии)
                // ...
            } else {
                // Проверка на переход к следующей стадии разрушения
                let health_percent = destructible.health / destructible.max_health;
                
                for (i, stage) in destructible.destruction_stages.iter().enumerate() {
                    if health_percent <= stage.health_threshold && i > destructible.current_stage {
                        destructible.current_stage = i;
                        
                        // Обновление визуальной модели
                        // ...
                        
                        // Эффекты перехода к новой стадии разрушения
                        // ...
                        
                        break;
                    }
                }
            }
        }
    }
}

/// Создает компонент погоды
pub fn create_weather(world: &mut World, weather_type: WeatherType, intensity: f32) -> hecs::Entity {
    let weather = WeatherComponent {
        weather_type,
        intensity,
        transition_time: 10.0,
        current_time: 0.0,
        target_weather: None,
    };
    
    world.spawn((weather,))
}

/// Создает компонент времени суток
pub fn create_time_of_day(world: &mut World, hour: f32, minute: f32) -> hecs::Entity {
    let time_of_day = TimeOfDayComponent {
        hour,
        minute,
        ..Default::default()
    };
    
    world.spawn((time_of_day,))
}

/// Создает разрушаемый объект
pub fn create_destructible_object(
    world: &mut World,
    position: Vec3,
    rotation: Quat,
    health: f32,
    object_type: EnvironmentObjectType,
) -> hecs::Entity {
    let transform = TransformComponent {
        position,
        rotation,
        ..Default::default()
    };
    
    let destructible = DestructibleComponent {
        health,
        max_health: health,
        destroyed: false,
        destruction_threshold: 0.1,
        destruction_stages: vec![
            DestructionStage {
                health_threshold: 0.7,
                mesh_id: 1,
                effects: vec!["smoke".to_string()],
                sounds: vec!["crack".to_string()],
            },
            DestructionStage {
                health_threshold: 0.3,
                mesh_id: 2,
                effects: vec!["smoke".to_string(), "sparks".to_string()],
                sounds: vec!["break".to_string()],
            },
        ],
        current_stage: 0,
    };
    
    let environment_object = EnvironmentObjectComponent {
        object_type,
        can_collide: true,
        is_static: true,
    };
    
    world.spawn((transform, destructible, environment_object))
} 