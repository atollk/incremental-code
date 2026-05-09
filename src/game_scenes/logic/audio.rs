use crate::game_state::{with_settings, with_settings_mut};
use include_dir::{Dir, include_dir};
use itertools::Itertools;
use rand::prelude::IteratorRandom;
use rand::{random_iter, random_range, rng};
use rodio::{MixerDeviceSink, Player};
use std::io::Cursor;
use std::ops::{Deref, DerefMut};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

pub struct AudioBackend {
    _sink: MixerDeviceSink,
    player: Player,
    bgm_silence: Option<Duration>,
}

const MUSIC_ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/music");

const STARTING_VOLUME: f32 = 0.01;

impl AudioBackend {
    pub fn new() -> anyhow::Result<Self> {
        let sink = rodio::DeviceSinkBuilder::open_default_sink()?;
        let player = Player::connect_new(sink.mixer());
        player.set_volume(STARTING_VOLUME);
        Ok(Self {
            _sink: sink,
            player,
            bgm_silence: None,
        })
    }

    pub fn tick(&mut self, time_delta: Duration) {
        match (self.player.empty(), self.bgm_silence) {
            (true, None) => {
                let silence = Duration::from_secs(random_range(10..20));
                self.bgm_silence = Some(silence);
            }
            (false, None) => {}
            (true, Some(_)) => {
                self.bgm_silence = if let Some(bgm_silence) = &mut self.bgm_silence {
                    bgm_silence.checked_sub(time_delta)
                } else {
                    None
                };
                if self.bgm_silence.is_none() {
                    if let Err(e) = self.start_bgm() {
                        log::error!("Error starting bgm: {:?}", e);
                    }
                }
            }
            (false, Some(_)) => {
                unreachable!()
            }
        }
    }

    pub fn start_bgm(&mut self) -> anyhow::Result<()> {
        with_settings(|settings| self.player.set_volume(settings.bgm_volume));
        self.bgm_silence = None;
        let [asset] = MUSIC_ASSETS
            .files()
            .sample(&mut rng(), 1)
            .into_iter()
            .collect_array()
            .unwrap();
        let source = rodio::Decoder::try_from(Cursor::new(asset.contents()))?;
        self.player.append(source);
        Ok(())
    }

    pub fn get_volume(&self) -> f32 {
        self.player.volume()
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.player.set_volume(volume);
        with_settings_mut(|settings| settings.bgm_volume = volume);
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
