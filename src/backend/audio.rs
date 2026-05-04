use rodio::{MixerDeviceSink, Player};
use std::io::Cursor;
use std::sync::{LazyLock, Mutex};

pub struct AudioBackend {
    sink: MixerDeviceSink,
    player: Player,
}

impl AudioBackend {
    pub fn new() -> anyhow::Result<Self> {
        let sink = rodio::DeviceSinkBuilder::open_default_sink()?;
        let player = Player::connect_new(sink.mixer());
        player.set_volume(0.7);
        Ok(Self { sink, player })
    }

    pub fn play(&mut self) -> anyhow::Result<()> {
        let bytes = include_bytes!("../../assets/purrplecat-tabula-rasa-360276.mp3");
        let source = rodio::Decoder::try_from(Cursor::new(bytes.as_slice()))?;
        self.player.append(source);
        Ok(())
    }
}

/// `None` if the audio device could not be opened (logs a warning, does not panic).
pub static AUDIO_BACKEND: LazyLock<Option<Mutex<AudioBackend>>> =
    LazyLock::new(|| match AudioBackend::new() {
        Ok(b) => Some(Mutex::new(b)),
        Err(e) => {
            log::warn!("Audio backend unavailable: {e}");
            None
        }
    });
