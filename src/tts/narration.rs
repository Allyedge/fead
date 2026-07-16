use std::collections::VecDeque;
use std::num::NonZero;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use rodio::buffer::SamplesBuffer;
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player};
use sherpa_onnx::GenerationConfig;
use tokio::sync::mpsc;

use super::text::NarrationUnit;
use super::TTS;

const LOOKAHEAD: usize = 3;
const START_AFTER: usize = 2;
const DEFAULT_SAMPLE_RATE: u32 = 24_000;
const POLL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum NarrationUiState {
    #[default]
    Idle,
    Preparing {
        current: usize,
        total: usize,
    },
    Playing {
        current: usize,
        total: usize,
    },
    Paused {
        current: usize,
        total: usize,
    },
    Buffering {
        current: usize,
        total: usize,
    },
    Completed,
    Error,
}

impl NarrationUiState {
    pub fn is_active(self) -> bool {
        matches!(
            self,
            Self::Preparing { .. }
                | Self::Playing { .. }
                | Self::Paused { .. }
                | Self::Buffering { .. }
        )
    }

    pub fn status_line(self) -> Option<String> {
        match self {
            Self::Idle => None,
            Self::Preparing { current, total } => {
                Some(format!("Preparing speech… {current}/{total}"))
            }
            Self::Playing { current, total } => {
                Some(format!("Playing {current}/{total}  ·  Space pause  ·  s stop"))
            }
            Self::Paused { current, total } => {
                Some(format!("Paused {current}/{total}  ·  Space resume  ·  s stop"))
            }
            Self::Buffering { current, total } => {
                Some(format!("Buffering {current}/{total}  ·  Space pause  ·  s stop"))
            }
            Self::Completed => Some("Finished reading.  ·  Space replay  ·  s reset".into()),
            Self::Error => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum NarrationEvent {
    State(NarrationUiState),
    Error(String),
}

enum Command {
    SetEngine(Option<Arc<TTS>>),
    Play { units: Vec<NarrationUnit> },
    TogglePause,
    Stop,
    Shutdown,
}

struct ReadyUnit {
    index: usize,
    samples: Vec<f32>,
    sample_rate: u32,
}

struct AudioOut {
    _stream: MixerDeviceSink,
    player: Player,
}

struct Session {
    id: u64,
    units: Vec<NarrationUnit>,
    total: usize,
    next_synth: usize,
    next_append: usize,
    ready: VecDeque<ReadyUnit>,
    cancel: Arc<AtomicBool>,
    paused: bool,
    started: bool,
    audio: Option<AudioOut>,
    sample_rate: u32,
    synth_in_flight: bool,
    last_state: Option<NarrationUiState>,
}

#[derive(Clone)]
pub struct NarrationHandle {
    cmd_tx: mpsc::UnboundedSender<Command>,
}

impl NarrationHandle {
    pub fn set_engine(&self, engine: Option<Arc<TTS>>) {
        let _ = self.cmd_tx.send(Command::SetEngine(engine));
    }

    pub fn play(&self, units: Vec<NarrationUnit>) {
        let _ = self.cmd_tx.send(Command::Play { units });
    }

    pub fn toggle_pause(&self) {
        let _ = self.cmd_tx.send(Command::TogglePause);
    }

    pub fn stop(&self) {
        let _ = self.cmd_tx.send(Command::Stop);
    }

    pub fn shutdown(&self) {
        let _ = self.cmd_tx.send(Command::Shutdown);
    }
}

pub fn spawn_narration() -> (NarrationHandle, mpsc::UnboundedReceiver<NarrationEvent>) {
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    tokio::spawn(run_controller(cmd_rx, event_tx));
    (NarrationHandle { cmd_tx }, event_rx)
}

async fn run_controller(
    mut cmd_rx: mpsc::UnboundedReceiver<Command>,
    event_tx: mpsc::UnboundedSender<NarrationEvent>,
) {
    let (synth_tx, mut synth_rx) = mpsc::unbounded_channel::<SynthResult>();
    let mut engine: Option<Arc<TTS>> = None;
    let mut session: Option<Session> = None;
    let mut next_session_id = 1u64;
    let mut interval = tokio::time::interval(POLL_INTERVAL);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            cmd = cmd_rx.recv() => {
                let Some(cmd) = cmd else { break; };
                match cmd {
                    Command::Shutdown => {
                        if let Some(active) = session.take() {
                            active.cancel.store(true, Ordering::SeqCst);
                            stop_audio(active.audio);
                        }
                        break;
                    }
                    Command::SetEngine(next) => {
                        engine = next;
                    }
                    Command::Stop => {
                        if let Some(active) = session.take() {
                            active.cancel.store(true, Ordering::SeqCst);
                            stop_audio(active.audio);
                        }
                        emit(&event_tx, NarrationEvent::State(NarrationUiState::Idle));
                    }
                    Command::TogglePause => {
                        if let Some(active) = session.as_mut() {
                            active.paused = !active.paused;
                            if let Some(audio) = active.audio.as_ref() {
                                if active.paused {
                                    audio.player.pause();
                                } else {
                                    audio.player.play();
                                    pump_ready(active);
                                }
                            }
                            emit_session_state(&event_tx, active);
                        }
                    }
                    Command::Play { units } => {
                        if let Some(active) = session.take() {
                            active.cancel.store(true, Ordering::SeqCst);
                            stop_audio(active.audio);
                        }

                        let Some(engine) = engine.clone() else {
                            emit(
                                &event_tx,
                                NarrationEvent::Error("TTS is not ready. Press t to set it up.".into()),
                            );
                            emit(&event_tx, NarrationEvent::State(NarrationUiState::Error));
                            continue;
                        };

                        if units.is_empty() {
                            emit(
                                &event_tx,
                                NarrationEvent::Error("No article content to read.".into()),
                            );
                            emit(&event_tx, NarrationEvent::State(NarrationUiState::Error));
                            continue;
                        }

                        let id = next_session_id;
                        next_session_id = next_session_id.wrapping_add(1).max(1);
                        let total = units.len();
                        let cancel = Arc::new(AtomicBool::new(false));
                        let mut active = Session {
                            id,
                            units,
                            total,
                            next_synth: 0,
                            next_append: 0,
                            ready: VecDeque::new(),
                            cancel: Arc::clone(&cancel),
                            paused: false,
                            started: false,
                            audio: None,
                            sample_rate: DEFAULT_SAMPLE_RATE,
                            synth_in_flight: false,
                            last_state: None,
                        };
                        emit_session_state(&event_tx, &mut active);
                        maybe_spawn_synth(Some(&mut active), &engine, &synth_tx);
                        session = Some(active);
                    }
                }
            }
            result = synth_rx.recv() => {
                let Some(result) = result else { break; };
                let Some(active) = session.as_mut() else { continue; };
                if result.session_id != active.id {
                    continue;
                }
                active.synth_in_flight = false;

                match result.outcome {
                    Ok(ready) => {
                        if let Some(rate) = ready.as_ref().map(|unit| unit.sample_rate) {
                            active.sample_rate = rate;
                        }
                        if let Some(ready) = ready {
                            active.ready.push_back(ready);
                        }
                        active.next_synth += 1;

                        if !active.started {
                            if ready_to_start(active) {
                                if let Err(error) = start_playback(active) {
                                    active.cancel.store(true, Ordering::SeqCst);
                                    stop_audio(active.audio.take());
                                    session = None;
                                    emit(&event_tx, NarrationEvent::Error(error));
                                    emit(&event_tx, NarrationEvent::State(NarrationUiState::Error));
                                    continue;
                                }
                            }
                        } else if !active.paused {
                            pump_if_needed(active);
                        }

                        emit_session_state(&event_tx, active);
                        if let Some(engine) = engine.clone() {
                            maybe_spawn_synth(session.as_mut(), &engine, &synth_tx);
                        }
                    }
                    Err(error) => {
                        active.cancel.store(true, Ordering::SeqCst);
                        stop_audio(active.audio.take());
                        session = None;
                        emit(&event_tx, NarrationEvent::Error(error));
                        emit(&event_tx, NarrationEvent::State(NarrationUiState::Error));
                    }
                }
            }
            _ = interval.tick() => {
                let Some(active) = session.as_mut() else { continue; };
                if !active.started {
                    if let Some(engine) = engine.clone() {
                        maybe_spawn_synth(Some(active), &engine, &synth_tx);
                    }
                    continue;
                }

                if !active.paused {
                    pump_if_needed(active);
                }

                let player_empty = active
                    .audio
                    .as_ref()
                    .map(|audio| audio.player.empty())
                    .unwrap_or(true);
                let synth_done = active.next_synth >= active.total && !active.synth_in_flight;

                if player_empty && active.ready.is_empty() && synth_done {
                    stop_audio(active.audio.take());
                    session = None;
                    emit(&event_tx, NarrationEvent::State(NarrationUiState::Completed));
                    continue;
                }

                emit_session_state(&event_tx, active);

                if let Some(engine) = engine.clone() {
                    maybe_spawn_synth(session.as_mut(), &engine, &synth_tx);
                }
            }
        }
    }
}

struct SynthResult {
    session_id: u64,
    outcome: Result<Option<ReadyUnit>, String>,
}

fn maybe_spawn_synth(
    session: Option<&mut Session>,
    engine: &Arc<TTS>,
    synth_tx: &mpsc::UnboundedSender<SynthResult>,
) {
    let Some(session) = session else { return; };
    if session.synth_in_flight {
        return;
    }
    if session.ready.len() >= LOOKAHEAD {
        return;
    }
    if session.next_synth >= session.total {
        return;
    }
    if session.cancel.load(Ordering::SeqCst) {
        return;
    }

    let index = session.next_synth;
    let unit = session.units[index].clone();
    let session_id = session.id;
    let cancel = Arc::clone(&session.cancel);
    let engine = Arc::clone(engine);
    let synth_tx = synth_tx.clone();
    session.synth_in_flight = true;

    tokio::task::spawn_blocking(move || {
        let outcome = synthesize_unit(engine, index, unit, cancel);
        let _ = synth_tx.send(SynthResult {
            session_id,
            outcome,
        });
    });
}

fn synthesize_unit(
    engine: Arc<TTS>,
    index: usize,
    unit: NarrationUnit,
    cancel: Arc<AtomicBool>,
) -> Result<Option<ReadyUnit>, String> {
    if cancel.load(Ordering::SeqCst) {
        return Ok(None);
    }

    match unit {
        NarrationUnit::Silence { ms } => {
            let samples = silence_samples(DEFAULT_SAMPLE_RATE, ms);
            Ok(Some(ReadyUnit {
                index,
                samples,
                sample_rate: DEFAULT_SAMPLE_RATE,
            }))
        }
        NarrationUnit::Speech(text) => {
            if text.trim().is_empty() {
                return Ok(None);
            }

            let gen_config = GenerationConfig {
                sid: 0,
                speed: 1.0,
                ..Default::default()
            };

            let cancel_flag = Arc::clone(&cancel);
            let audio = engine
                .engine
                .generate_with_config(
                    &text,
                    &gen_config,
                    Some(move |_samples: &[f32], _progress: f32| -> bool {
                        !cancel_flag.load(Ordering::SeqCst)
                    }),
                )
                .ok_or_else(|| "speech generation failed".to_string())?;

            if cancel.load(Ordering::SeqCst) {
                return Ok(None);
            }

            let samples = audio.samples().to_vec();
            let sample_rate = audio.sample_rate();
            drop(audio);

            if samples.is_empty() {
                return Ok(None);
            }
            if sample_rate <= 0 {
                return Err("invalid sample rate".into());
            }

            Ok(Some(ReadyUnit {
                index,
                samples,
                sample_rate: sample_rate as u32,
            }))
        }
    }
}

fn silence_samples(sample_rate: u32, ms: u32) -> Vec<f32> {
    let count = (sample_rate as u64 * ms as u64 / 1000) as usize;
    vec![0.0; count.max(1)]
}

fn ready_to_start(session: &Session) -> bool {
    if session.ready.is_empty() {
        return false;
    }
    let remaining = session.total.saturating_sub(session.next_synth);
    session.ready.len() >= START_AFTER || remaining == 0
}

fn start_playback(session: &mut Session) -> Result<(), String> {
    let mut stream = DeviceSinkBuilder::open_default_sink().map_err(|e| e.to_string())?;
    stream.log_on_drop(false);
    let player = Player::connect_new(stream.mixer());
    if session.paused {
        player.pause();
    }
    session.audio = Some(AudioOut {
        _stream: stream,
        player,
    });
    session.started = true;
    pump_ready(session);
    Ok(())
}

fn pump_if_needed(session: &mut Session) {
    let player_empty = session
        .audio
        .as_ref()
        .map(|audio| audio.player.empty())
        .unwrap_or(true);

    if player_empty {
        let remaining = session.total.saturating_sub(session.next_synth);
        let all_remaining_ready = remaining == 0 && !session.synth_in_flight;
        if session.ready.len() < START_AFTER && !all_remaining_ready {
            return;
        }
        if session.ready.is_empty() {
            return;
        }
    }

    pump_ready(session);
}

fn pump_ready(session: &mut Session) {
    let Some(audio) = session.audio.as_ref() else {
        return;
    };
    while let Some(unit) = session.ready.pop_front() {
        session.sample_rate = unit.sample_rate;
        if let Err(error) = append_unit(audio, &unit) {
            session.ready.push_front(unit);
            let _ = error;
            break;
        }
        session.next_append = unit.index + 1;
    }
}

fn append_unit(audio: &AudioOut, unit: &ReadyUnit) -> Result<(), String> {
    let sample_rate =
        NonZero::new(unit.sample_rate).ok_or_else(|| "invalid sample rate".to_string())?;
    let channels = NonZero::new(1u16).expect("channels");
    audio.player.append(SamplesBuffer::new(
        channels,
        sample_rate,
        unit.samples.clone(),
    ));
    Ok(())
}

fn stop_audio(audio: Option<AudioOut>) {
    if let Some(audio) = audio {
        audio.player.stop();
        drop(audio);
    }
}

fn progress_current(session: &Session) -> usize {
    if session.total == 0 {
        return 0;
    }
    let current = session.next_append.max(session.next_synth.min(session.total));
    current.clamp(1, session.total)
}

fn emit_session_state(event_tx: &mpsc::UnboundedSender<NarrationEvent>, session: &mut Session) {
    let current = progress_current(session);
    let total = session.total;
    let state = if !session.started {
        NarrationUiState::Preparing { current, total }
    } else if session.paused {
        NarrationUiState::Paused { current, total }
    } else {
        let player_empty = session
            .audio
            .as_ref()
            .map(|audio| audio.player.empty())
            .unwrap_or(true);
        if player_empty && session.ready.is_empty() && session.next_synth < session.total {
            NarrationUiState::Buffering { current, total }
        } else {
            NarrationUiState::Playing { current, total }
        }
    };
    if session.last_state != Some(state) {
        session.last_state = Some(state);
        emit(event_tx, NarrationEvent::State(state));
    }
}

fn emit(event_tx: &mpsc::UnboundedSender<NarrationEvent>, event: NarrationEvent) {
    let _ = event_tx.send(event);
}
