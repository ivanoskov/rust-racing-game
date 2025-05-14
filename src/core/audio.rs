use crate::core::ecs::{Resource};
use hecs::World;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Система аудио
pub struct AudioSystem {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sound_library: HashMap<String, Arc<Vec<u8>>>,
    sinks: HashMap<String, Arc<Mutex<Sink>>>,
    music_sink: Option<Arc<Mutex<Sink>>>,
    current_music: Option<String>,
    volume: f32,
}

impl AudioSystem {
    pub fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        
        Self {
            _stream: stream,
            stream_handle,
            sound_library: HashMap::new(),
            sinks: HashMap::new(),
            music_sink: None,
            current_music: None,
            volume: 1.0,
        }
    }
    
    /// Загрузка звука из файла
    pub fn load_sound(&mut self, name: &str, path: &Path) -> Result<(), String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let mut buffer = Vec::new();
        let mut reader = BufReader::new(file);
        std::io::Read::read_to_end(&mut reader, &mut buffer).map_err(|e| e.to_string())?;
        
        self.sound_library.insert(name.to_string(), Arc::new(buffer));
        Ok(())
    }
    
    /// Воспроизведение звука
    pub fn play_sound(&mut self, name: &str, volume: f32, looping: bool) -> Result<String, String> {
        let sound_data = self.sound_library
            .get(name)
            .ok_or_else(|| format!("Sound {} not found", name))?
            .clone();
        
        let sink = Sink::try_new(&self.stream_handle).map_err(|e| e.to_string())?;
        sink.set_volume(volume * self.volume);
        
        let sound_cursor = std::io::Cursor::new(sound_data.to_vec());
        let source = Decoder::new(sound_cursor).map_err(|e| e.to_string())?;
        
        if looping {
            sink.append(source.repeat_infinite());
        } else {
            sink.append(source);
        }
        
        let id = format!("{}_{}", name, Uuid::new_v4().to_string());
        self.sinks.insert(id.clone(), Arc::new(Mutex::new(sink)));
        
        Ok(id)
    }
    
    /// Остановка звука по ID
    pub fn stop_sound(&mut self, id: &str) -> Result<(), String> {
        if let Some(sink) = self.sinks.remove(id) {
            let sink = sink.lock().map_err(|e| e.to_string())?;
            sink.stop();
        }
        Ok(())
    }
    
    /// Установка громкости звука по ID
    pub fn set_sound_volume(&mut self, id: &str, volume: f32) -> Result<(), String> {
        if let Some(sink) = self.sinks.get(id) {
            let sink = sink.lock().map_err(|e| e.to_string())?;
            sink.set_volume(volume * self.volume);
        }
        Ok(())
    }
    
    /// Воспроизведение музыки с возможностью переключения
    pub fn play_music(&mut self, name: &str, volume: f32) -> Result<(), String> {
        // Если музыка уже играет и это та же самая музыка, просто меняем громкость
        if let Some(current) = &self.current_music {
            if current == name {
                if let Some(sink) = &self.music_sink {
                    let sink = sink.lock().map_err(|e| e.to_string())?;
                    sink.set_volume(volume * self.volume);
                    return Ok(());
                }
            }
        }
        
        // Остановить текущую музыку, если она играет
        if let Some(sink) = &self.music_sink {
            let sink = sink.lock().map_err(|e| e.to_string())?;
            sink.stop();
        }
        
        // Воспроизвести новую музыку
        let sound_data = self.sound_library
            .get(name)
            .ok_or_else(|| format!("Music {} not found", name))?
            .clone();
        
        let sink = Sink::try_new(&self.stream_handle).map_err(|e| e.to_string())?;
        sink.set_volume(volume * self.volume);
        
        let sound_cursor = std::io::Cursor::new(sound_data.to_vec());
        let source = Decoder::new(sound_cursor).map_err(|e| e.to_string())?;
        sink.append(source.repeat_infinite());
        
        self.music_sink = Some(Arc::new(Mutex::new(sink)));
        self.current_music = Some(name.to_string());
        
        Ok(())
    }
    
    /// Остановка музыки
    pub fn stop_music(&mut self) -> Result<(), String> {
        if let Some(sink) = &self.music_sink {
            let sink = sink.lock().map_err(|e| e.to_string())?;
            sink.stop();
        }
        self.music_sink = None;
        self.current_music = None;
        Ok(())
    }
    
    /// Установка общей громкости
    pub fn set_master_volume(&mut self, volume: f32) {
        self.volume = volume;
        
        // Обновляем громкость для всех звуков и музыки
        for sink in self.sinks.values() {
            if let Ok(sink) = sink.lock() {
                sink.set_volume(sink.volume() * self.volume);
            }
        }
        
        if let Some(sink) = &self.music_sink {
            if let Ok(sink) = sink.lock() {
                sink.set_volume(sink.volume() * self.volume);
            }
        }
    }
    
    /// Очистка неактивных звуков
    pub fn cleanup(&mut self) {
        let mut to_remove = Vec::new();
        
        for (id, sink) in &self.sinks {
            if let Ok(sink) = sink.lock() {
                if sink.empty() {
                    to_remove.push(id.clone());
                }
            }
        }
        
        for id in to_remove {
            self.sinks.remove(&id);
        }
    }

    /// Обработка аудио-событий и компонентов звуковых источников
    pub fn process(&mut self, world: &mut World, _delta_time: f32) {
        // Очистка неактивных звуков
        self.cleanup();
        
        // Получаем ресурс с событиями аудио (если есть)
        let audio_events = world.query_mut::<&mut Resource<Vec<AudioEvent>>>()
            .into_iter()
            .next()
            .map(|(_, res)| &mut res.0);
        
        if let Some(events) = audio_events {
            for event in events.drain(..) {
                match event {
                    AudioEvent::PlaySound { name, volume, looping } => {
                        let _ = self.play_sound(&name, volume, looping);
                    }
                    AudioEvent::StopSound { id } => {
                        let _ = self.stop_sound(&id);
                    }
                    AudioEvent::SetSoundVolume { id, volume } => {
                        let _ = self.set_sound_volume(&id, volume);
                    }
                    AudioEvent::PlayMusic { name, volume } => {
                        let _ = self.play_music(&name, volume);
                    }
                    AudioEvent::StopMusic => {
                        let _ = self.stop_music();
                    }
                    AudioEvent::SetMasterVolume { volume } => {
                        self.set_master_volume(volume);
                    }
                }
            }
        }
        
        // Обработка компонентов звуковых источников
        // Для простоты пока не реализуем 3D-звук, только базовые функции
        for (_, audio_source) in world.query_mut::<&mut AudioSourceComponent>() {
            if audio_source.sound_id.is_none() && !audio_source.sound_name.is_empty() {
                // Воспроизвести звук, если он еще не воспроизводится
                if let Ok(id) = self.play_sound(&audio_source.sound_name, audio_source.volume, audio_source.looping) {
                    audio_source.sound_id = Some(id);
                }
            }
        }
    }
}

/// Компонент звукового источника, связанный с сущностью
pub struct AudioSourceComponent {
    pub sound_id: Option<String>,
    pub sound_name: String,
    pub volume: f32,
    pub pitch: f32,
    pub spatial: bool,
    pub min_distance: f32,
    pub max_distance: f32,
    pub looping: bool,
}

impl Default for AudioSourceComponent {
    fn default() -> Self {
        Self {
            sound_id: None,
            sound_name: String::new(),
            volume: 1.0,
            pitch: 1.0,
            spatial: true,
            min_distance: 1.0,
            max_distance: 50.0,
            looping: false,
        }
    }
}

/// Событие звука
pub enum AudioEvent {
    PlaySound { name: String, volume: f32, looping: bool },
    StopSound { id: String },
    SetSoundVolume { id: String, volume: f32 },
    PlayMusic { name: String, volume: f32 },
    StopMusic,
    SetMasterVolume { volume: f32 },
} 