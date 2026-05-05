use rodio::{MixerDeviceSink, Player};
use std::io::Cursor;
use std::ops::{Deref, DerefMut};
use std::sync::{LazyLock, Mutex};

pub struct AudioBackend {
    _sink: MixerDeviceSink,
    player: Player,
}

impl AudioBackend {
    pub fn new() -> anyhow::Result<Self> {
        let sink = rodio::DeviceSinkBuilder::open_default_sink()?;
        let player = Player::connect_new(sink.mixer());
        player.set_volume(0.7);
        Ok(Self {
            _sink: sink,
            player,
        })
    }

    pub fn start_bgm(&mut self) -> anyhow::Result<()> {
        let bytes = include_bytes!("../../assets/purrplecat-tabula-rasa-360276.mp3");
        let source = rodio::Decoder::try_from(Cursor::new(bytes.as_slice()))?;
        self.player.append(source);
        Ok(())
    }

    pub fn get_volume(&self) -> f32 {
        self.player.volume()
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.player.set_volume(volume);
    }
}

pub fn with_audio_backend<T>(f: impl FnOnce(&mut AudioBackend) -> T) -> Option<T> {
    let lock = AUDIO_BACKEND.as_ref()?.lock();
    Some(f(&mut *lock.ok()?))
}

/// `None` if the audio device could not be opened (logs a warning, does not panic).
static AUDIO_BACKEND: LazyLock<Option<Mutex<AudioBackend>>> =
    LazyLock::new(|| match AudioBackend::new() {
        Ok(b) => Some(Mutex::new(b)),
        Err(e) => {
            log::warn!("Audio backend unavailable: {e}");
            None
        }
    });
