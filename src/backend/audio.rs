use crate::widgets::tree::{Tree, TreeItem, TreeState};
use ouroboros::self_referencing;
use rodio::{MixerDeviceSink, Player};
use std::cell::LazyCell;
use std::error::Error;
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
        let file = std::fs::File::open("assets/purrplecat-tabula-rasa-360276.mp3")?;
        let source = rodio::Decoder::try_from(file)?;
        self.player.append(source);
        Ok(())
    }
}

pub static AUDIO_BACKEND: LazyLock<Mutex<AudioBackend>> =
    LazyLock::new(|| Mutex::new(AudioBackend::new().unwrap()));
