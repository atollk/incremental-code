use crate::game_scenes::base::TickerMut;
use crate::game_state::{with_settings, with_settings_mut};
use crate::global_variable;
use include_dir::{Dir, include_dir};
use itertools::Itertools;
use rand::prelude::IteratorRandom;
use rand::{random_range, rng};
use rodio::{MixerDeviceSink, Player};
use std::io::Cursor;
use std::time::Duration;
use tap::TapFallible;

struct AudioBackendInner {
    #[allow(unused)]
    sink: MixerDeviceSink,
    player: Player,
    bgm_silence: Option<Duration>,
}

impl AudioBackendInner {
    fn new() -> anyhow::Result<Self> {
        let sink = rodio::DeviceSinkBuilder::open_default_sink()?;
        let player = Player::connect_new(sink.mixer());
        player.set_volume(STARTING_VOLUME);
        Ok(AudioBackendInner {
            sink,
            player,
            bgm_silence: None,
        })
    }
}

pub struct AudioBackend {
    inner: Option<AudioBackendInner>,
}

const MUSIC_ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/music");

const STARTING_VOLUME: f32 = 0.01;

impl AudioBackend {
    pub fn new() -> Self {
        let inner = AudioBackendInner::new()
            .tap_err(|err| log::warn!("Failed to open sound device: {}", err))
            .ok();
        Self { inner }
    }

    pub fn start_bgm_loop(&mut self) -> anyhow::Result<()> {
        let Some(inner) = &mut self.inner else {
            return Ok(());
        };
        with_settings(|settings| inner.player.set_volume(settings.bgm_volume));
        if inner.bgm_silence.is_none() && inner.player.empty() {
            self.start_bgm()?;
        }
        Ok(())
    }

    fn start_bgm(&mut self) -> anyhow::Result<()> {
        let Some(inner) = &mut self.inner else {
            return Ok(());
        };
        inner.bgm_silence = None;
        let [asset] = MUSIC_ASSETS
            .files()
            .sample(&mut rng(), 1)
            .into_iter()
            .collect_array()
            .unwrap();
        let source = rodio::Decoder::try_from(Cursor::new(asset.contents()))?;
        inner.player.append(source);
        Ok(())
    }

    pub fn stop_bgm(&mut self) {
        let Some(inner) = &mut self.inner else { return };
        inner.player.clear();
    }

    pub fn get_volume(&self) -> f32 {
        let Some(inner) = &self.inner else { return 0. };
        inner.player.volume()
    }

    pub fn set_volume(&mut self, volume: f32) {
        let Some(inner) = &mut self.inner else { return };
        inner.player.set_volume(volume);
        with_settings_mut(|settings| settings.bgm_volume = volume);
    }
}

impl Default for AudioBackend {
    fn default() -> Self {
        Self::new()
    }
}

fn silence_between_songs() -> Duration {
    Duration::from_secs(random_range(5..10))
}

impl TickerMut for AudioBackend {
    fn tick(&mut self, elapsed: Duration) {
        let Some(inner) = &mut self.inner else { return };
        match (inner.player.empty(), inner.bgm_silence) {
            (true, None) => {
                inner.bgm_silence = Some(silence_between_songs());
            }
            (false, None) => {}
            (true, Some(_)) => {
                inner.bgm_silence = if let Some(bgm_silence) = &mut inner.bgm_silence {
                    bgm_silence.checked_sub(elapsed)
                } else {
                    None
                };
                if inner.bgm_silence.is_none() {
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
}

global_variable!(audio_backend, AudioBackend);
