use crate::core::ecs::{EventQueue, Resource};
use gilrs::{Gilrs, Button};
use hecs::World;
use std::collections::HashMap;
use winit::{
    event::*,
    keyboard::{KeyCode, PhysicalKey}
};
use winit_input_helper::WinitInputHelper;

/// Типы событий ввода
#[derive(Debug, Clone)]
pub enum InputEvent {
    KeyPressed(KeyCode),
    KeyReleased(KeyCode),
    MouseMoved(f32, f32),
    MousePressed(MouseButton),
    MouseReleased(MouseButton),
    MouseWheel(f32),
    GamepadButton(usize, Button, bool),
    GamepadAxis(usize, u32, f32),
    GamepadConnected(usize),
    GamepadDisconnected(usize),
}

/// Тип устройства ввода
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputDevice {
    Keyboard,
    Mouse,
    Gamepad(usize),
}

/// Действия ввода для игры
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputAction {
    Accelerate,
    Brake,
    SteerLeft,
    SteerRight,
    Handbrake,
    ShiftUp,
    ShiftDown,
    ToggleCamera,
    Pause,
    // Добавьте другие действия по мере необходимости
}

/// Система ввода
pub struct InputSystem {
    input_helper: WinitInputHelper,
    gilrs: Gilrs,
    action_bindings: HashMap<InputAction, Vec<InputBinding>>,
    action_states: HashMap<InputAction, f32>,
}

/// Привязка ввода к действию
#[derive(Debug, Clone)]
pub struct InputBinding {
    pub device: InputDevice,
    pub input_type: InputType,
    pub value_scale: f32,
}

/// Тип ввода
#[derive(Debug, Clone)]
pub enum InputType {
    Key(KeyCode),
    MouseButton(MouseButton),
    MouseAxis(MouseAxis),
    GamepadButton(Button),
    GamepadAxis(u32, AxisDirection),
}

/// Направление оси
#[derive(Debug, Clone, Copy)]
pub enum AxisDirection {
    Positive,
    Negative,
    Both,
}

/// Оси мыши
#[derive(Debug, Clone, Copy)]
pub enum MouseAxis {
    X,
    Y,
    ScrollWheel,
}

impl InputSystem {
    pub fn new() -> Self {
        let gilrs = Gilrs::new().unwrap_or_else(|_| {
            eprintln!("Failed to init gilrs, gamepad support disabled");
            Gilrs::new().expect("Failed to initialize gilrs")
        });

        let mut system = Self {
            input_helper: WinitInputHelper::new(),
            gilrs,
            action_bindings: HashMap::new(),
            action_states: HashMap::new(),
        };

        system.setup_default_bindings();
        system
    }

    fn setup_default_bindings(&mut self) {
        // Клавиатура
        self.bind_action(
            InputAction::Accelerate,
            InputBinding {
                device: InputDevice::Keyboard,
                input_type: InputType::Key(KeyCode::KeyW),
                value_scale: 1.0,
            },
        );

        self.bind_action(
            InputAction::Brake,
            InputBinding {
                device: InputDevice::Keyboard,
                input_type: InputType::Key(KeyCode::KeyS),
                value_scale: 1.0,
            },
        );

        self.bind_action(
            InputAction::SteerLeft,
            InputBinding {
                device: InputDevice::Keyboard,
                input_type: InputType::Key(KeyCode::KeyA),
                value_scale: 1.0,
            },
        );

        self.bind_action(
            InputAction::SteerRight,
            InputBinding {
                device: InputDevice::Keyboard,
                input_type: InputType::Key(KeyCode::KeyD),
                value_scale: 1.0,
            },
        );

        // Геймпад (пример)
        self.bind_action(
            InputAction::Accelerate,
            InputBinding {
                device: InputDevice::Gamepad(0),
                input_type: InputType::GamepadAxis(0, AxisDirection::Positive), // Правый триггер
                value_scale: 1.0,
            },
        );

        self.bind_action(
            InputAction::Brake,
            InputBinding {
                device: InputDevice::Gamepad(0),
                input_type: InputType::GamepadAxis(1, AxisDirection::Positive), // Левый триггер
                value_scale: 1.0,
            },
        );

        self.bind_action(
            InputAction::SteerLeft,
            InputBinding {
                device: InputDevice::Gamepad(0),
                input_type: InputType::GamepadAxis(2, AxisDirection::Negative), // Левый стик X-
                value_scale: 1.0,
            },
        );

        self.bind_action(
            InputAction::SteerRight,
            InputBinding {
                device: InputDevice::Gamepad(0),
                input_type: InputType::GamepadAxis(2, AxisDirection::Positive), // Левый стик X+
                value_scale: 1.0,
            },
        );
    }

    pub fn bind_action(&mut self, action: InputAction, binding: InputBinding) {
        self.action_bindings
            .entry(action)
            .or_insert_with(Vec::new)
            .push(binding);
    }

    pub fn get_action_value(&self, action: InputAction) -> f32 {
        *self.action_states.get(&action).unwrap_or(&0.0)
    }

    pub fn is_action_pressed(&self, action: InputAction) -> bool {
        self.get_action_value(action) > 0.5
    }

    pub fn handle_event(&mut self, event: &winit::event::Event<()>, input_events: &mut EventQueue<InputEvent>) {
        // Обработка событий winit
        if let Event::WindowEvent { event, .. } = event {
            if let Some(key_code) = self.get_key_code_from_event(event) {
                match event {
                    WindowEvent::KeyboardInput { 
                        event: KeyEvent { state: ElementState::Pressed, .. }, ..
                    } => {
                        input_events.publish(InputEvent::KeyPressed(key_code));
                    },
                    WindowEvent::KeyboardInput { 
                        event: KeyEvent { state: ElementState::Released, .. }, ..
                    } => {
                        input_events.publish(InputEvent::KeyReleased(key_code));
                    },
                    _ => {}
                }
            }
            
            // Обработка мыши упрощена
            if let WindowEvent::MouseInput { state, button, .. } = event {
                match state {
                    ElementState::Pressed => input_events.publish(InputEvent::MousePressed(*button)),
                    ElementState::Released => input_events.publish(InputEvent::MouseReleased(*button)),
                }
            }
            
            // Другие события мыши и клавиатуры можно добавить по необходимости
        }
        
        // Обработка событий геймпада - упрощена из-за изменений API
        while let Some(_gilrs_event) = self.gilrs.next_event() {
            // Здесь можно добавить обработку событий геймпада
            // в соответствии с обновленным API gilrs
        }
    }
    
    fn get_key_code_from_event(&self, event: &WindowEvent) -> Option<KeyCode> {
        if let WindowEvent::KeyboardInput { 
            event: KeyEvent { physical_key: PhysicalKey::Code(key_code), .. }, ..
        } = event {
            return Some(*key_code);
        }
        None
    }

    fn update_action_states(&mut self, input_events: &mut EventQueue<InputEvent>) {
        // Обнуляем состояния действий для нажатия и отпускания
        // Для осей и других аналоговых вводов сохраняем состояние
        
        // Временно извлекаем все события
        let mut events = Vec::new();
        input_events.consume(|event| events.push(event.clone()));
        
        // Обрабатываем события ввода и обновляем состояния действий
        for event in events {
            match event {
                InputEvent::KeyPressed(key) => {
                    self.update_bindings_for_key_or_button(
                        key, 
                        1.0, 
                        |input_type| if let InputType::Key(k) = input_type { *k == key } else { false }
                    );
                }
                InputEvent::KeyReleased(key) => {
                    self.update_bindings_for_key_or_button(
                        key, 
                        0.0, 
                        |input_type| if let InputType::Key(k) = input_type { *k == key } else { false }
                    );
                }
                InputEvent::MousePressed(button) => {
                    self.update_bindings_for_key_or_button(
                        button, 
                        1.0, 
                        |input_type| if let InputType::MouseButton(b) = input_type { *b == button } else { false }
                    );
                }
                InputEvent::MouseReleased(button) => {
                    self.update_bindings_for_key_or_button(
                        button, 
                        0.0, 
                        |input_type| if let InputType::MouseButton(b) = input_type { *b == button } else { false }
                    );
                }
                InputEvent::GamepadButton(_id, button, pressed) => {
                    let value = if pressed { 1.0 } else { 0.0 };
                    self.update_bindings_for_key_or_button(
                        button, 
                        value, 
                        |input_type| if let InputType::GamepadButton(b) = input_type { *b == button } else { false }
                    );
                }
                InputEvent::GamepadAxis(id, axis, value) => {
                    // Обновить привязки для осей геймпада
                    self.update_axis_bindings(id, axis, value);
                }
                // Обработка других типов событий...
                _ => {}
            }
        }
    }

    fn update_bindings_for_key_or_button<T: std::fmt::Debug + Copy>(
        &mut self,
        _input: T,
        value: f32,
        predicate: impl Fn(&InputType) -> bool,
    ) {
        for (action, bindings) in &self.action_bindings {
            for binding in bindings {
                if predicate(&binding.input_type) {
                    let scaled_value = value * binding.value_scale;
                    self.action_states.insert(*action, scaled_value);
                }
            }
        }
    }

    fn update_axis_bindings(&mut self, gamepad_id: usize, axis_id: u32, value: f32) {
        for (action, bindings) in &self.action_bindings {
            for binding in bindings {
                if let InputType::GamepadAxis(axis, direction) = &binding.input_type {
                    if *axis == axis_id && binding.device == InputDevice::Gamepad(gamepad_id) {
                        let processed_value = match direction {
                            AxisDirection::Positive => value.max(0.0),
                            AxisDirection::Negative => (-value).max(0.0),
                            AxisDirection::Both => value.abs(),
                        };
                        
                        let scaled_value = processed_value * binding.value_scale;
                        self.action_states.insert(*action, scaled_value);
                    }
                }
            }
        }
    }

    // Добавим публичный метод process
    pub fn process(&mut self, world: &mut World, _delta_time: f32) {
        let input_events = world
            .query_mut::<&mut Resource<EventQueue<InputEvent>>>()
            .into_iter()
            .next()
            .map(|(_, res)| &mut res.0);
        
        // Если очереди событий нет, создаем ее
        if input_events.is_none() {
            let resource = Resource(EventQueue::<InputEvent>::new());
            world.spawn((resource,));
            return;
        }
        
        let input_events = input_events.unwrap();
        
        // Обновляем состояния действий на основе событий
        self.update_action_states(input_events);
        
        // Заполняем ресурс с состояниями действий в ECS
        let action_states = self.action_states.clone();
        
        let action_resource = world
            .query_mut::<&mut Resource<HashMap<InputAction, f32>>>()
            .into_iter()
            .next()
            .map(|(_, res)| &mut res.0);
        
        if let Some(action_resource) = action_resource {
            *action_resource = action_states;
        } else {
            let resource = Resource(action_states);
            world.spawn((resource,));
        }
    }
} 