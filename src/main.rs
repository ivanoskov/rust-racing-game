mod core;
mod game_world;
mod gameplay;
mod ui;
mod network;

use core::{
    Engine,
    input::InputSystem,
    audio::AudioSystem,
    renderer::{RenderSystem, RenderComponent, CameraComponent},
    ecs::{Resource, EventQueue},
    input::InputEvent,
};

use game_world::{
    GameWorldManager,
    car::create_car_entity,
    track::create_simple_track,
    environment::{create_time_of_day, create_weather, WeatherType},
};

use glam::{Vec3, Quat};
use winit::{
    event::*,
    event_loop::{EventLoopBuilder},
    window::WindowBuilder,
};

use std::time::{Instant, Duration};

fn main() {
    // Запуск асинхронного кода в блокирующем контексте
    pollster::block_on(run());
}

struct WindowState<'a> {
    window: &'a winit::window::Window,
    render_system: RenderSystem<'a>,
}

async fn run() {
    // Настройка логгера
    env_logger::init();
    
    // Создание event loop и окна
    let event_loop = EventLoopBuilder::<()>::new().build().unwrap();
    let window = WindowBuilder::new()
        .with_title("Racing Simulator")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)
        .unwrap();
    
    // Создание системы рендеринга с инициализацией графического контекста
    let render_system = RenderSystem::new(&window).await;
    
    // Создание состояния окна
    let mut window_state = WindowState {
        window: &window,
        render_system,
    };
    
    // Создание основных систем
    let mut engine = Engine::new();
    let mut input_system = InputSystem::new();
    let mut audio_system = AudioSystem::new();
    
    // Создание и инициализация игрового мира
    let mut game_world_manager = GameWorldManager::new();
    
    // Регистрация систем в ECS
    // Все системы теперь используем напрямую
    
    // Регистрация систем из GameWorldManager
    game_world_manager.register_systems(&mut engine.ecs_manager);
    
    // Инициализация физического мира
    game_world_manager.initialize_physics(&mut engine.ecs_manager);
    
    // Создание игрового мира
    create_game_world(&mut engine);
    
    // Время для расчета дельты
    let mut last_update_time = Instant::now();
    let target_frame_time = Duration::from_secs_f32(1.0 / 60.0); // 60 FPS
    
    // События ввода для передачи системе ввода
    let input_events = Resource(EventQueue::<InputEvent>::new());
    engine.ecs_manager.create_entity((input_events,));
    
    // Главный цикл
    let _ = event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window_state.window.id() => match event {
                WindowEvent::CloseRequested => {
                    elwt.exit();
                },
                WindowEvent::Resized(physical_size) => {
                    // Вызываем resize напрямую
                    window_state.render_system.resize(*physical_size);
                },
                WindowEvent::ScaleFactorChanged { .. } => {
                    // Обработка изменения масштабирования
                },
                _ => {}
            },
            Event::AboutToWait => {
                // Выполнение игрового цикла (ранее MainEventsCleared)
                let current_time = Instant::now();
                let delta_time = current_time.duration_since(last_update_time).as_secs_f32();
                last_update_time = current_time;
                
                // Обновление логики
                engine.update(delta_time);
                
                // Обновление систем напрямую
                input_system.process(&mut engine.ecs_manager.world, delta_time);
                game_world_manager.physics_system.process(&mut engine.ecs_manager.world, delta_time);
                audio_system.process(&mut engine.ecs_manager.world, delta_time);
                
                // Обновление рендера напрямую вызывая метод render
                window_state.render_system.render(&engine.ecs_manager.world, delta_time);
                
                // Обработка времени кадра для стабильного FPS
                let frame_time = current_time.elapsed();
                if frame_time < target_frame_time {
                    std::thread::sleep(target_frame_time - frame_time);
                }
                
                // Перерисовка
                window_state.window.request_redraw();
            },
            Event::WindowEvent { 
                event: WindowEvent::RedrawRequested,
                .. 
            } => {
                // Здесь может быть дополнительная логика для рендеринга
            },
            _ => {}
        }
    });
}

/// Создание и инициализация игрового мира
fn create_game_world(engine: &mut Engine) {
    // Создаем трассу
    let track_entity = create_simple_track(&mut engine.ecs_manager.world, 1000.0, 10.0);
    
    // Создаем автомобиль
    let car_entity = create_car_entity(
        &mut engine.ecs_manager.world, 
        "SportsCar", 
        Vec3::new(0.0, 0.5, 0.0), 
        Quat::IDENTITY
    );
    
    // Создаем компоненты окружения
    let weather_entity = create_weather(
        &mut engine.ecs_manager.world,
        WeatherType::Clear,
        0.0
    );
    
    let time_entity = create_time_of_day(
        &mut engine.ecs_manager.world,
        12.0, // Полдень
        0.0   // 0 минут
    );
    
    // Добавляем простой куб для визуализации дороги
    let mut render_system = RenderSystem::create_resource_manager();
    
    // Создаем меш куба для дороги
    let road_mesh_id = render_system.add_simple_cube();
    
    // Создаем материал для дороги
    let road_material_id = render_system.add_basic_material([0.3, 0.3, 0.3, 1.0]); // Серый цвет

    // Добавляем компонент рендеринга к трассе
    let road_render = RenderComponent {
        mesh_id: road_mesh_id,
        material_id: road_material_id,
        visible: true,
        scale: Vec3::new(10.0, 0.1, 1000.0), // Длинная, плоская дорога
    };
    engine.ecs_manager.world.insert_one(track_entity, road_render).unwrap();
    
    // Создаем меш для автомобиля (упрощенный)
    let car_mesh_id = render_system.add_simple_cube();
    
    // Создаем материал для автомобиля
    let car_material_id = render_system.add_basic_material([1.0, 0.0, 0.0, 1.0]); // Красный цвет
    
    // Добавляем компонент рендеринга к автомобилю
    let car_render = RenderComponent {
        mesh_id: car_mesh_id,
        material_id: car_material_id,
        visible: true,
        scale: Vec3::new(2.0, 1.0, 4.0), // Масштаб автомобиля
    };
    engine.ecs_manager.world.insert_one(car_entity, car_render).unwrap();
    
    // Добавляем ресурс менеджера рендеринга в мир
    engine.ecs_manager.world.spawn((Resource(render_system),));
    
    // Добавляем камеру
    let camera = CameraComponent {
        position: Vec3::new(0.0, 5.0, -10.0),
        target: Vec3::new(0.0, 0.0, 0.0),
        up: Vec3::new(0.0, 1.0, 0.0),
        aspect: 16.0 / 9.0, // Стандартное соотношение сторон
        fovy: 45.0 * std::f32::consts::PI / 180.0, // 45 градусов в радианах
        znear: 0.1,
        zfar: 1000.0,
    };
    
    engine.ecs_manager.world.spawn((camera,));
}