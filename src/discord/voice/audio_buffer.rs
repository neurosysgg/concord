use std::sync::mpsc::{Receiver as StdReceiver, TryRecvError};

use super::{DISCORD_VOICE_CHANNELS, DISCORD_VOICE_SAMPLE_RATE, VOICE_OUTPUT_UNDERRUN_FADE_MILLIS};

pub(super) struct VoiceAudioBuffer {
    samples_rx: StdReceiver<Vec<f32>>,
    current: Vec<f32>,
    offset: usize,
    output_sample_rate: u32,
    source_position: f64,
    last_frame: [f32; 2],
    fade_remaining_frames: usize,
    fade_total_frames: usize,
}

impl VoiceAudioBuffer {
    pub(super) fn new(samples_rx: StdReceiver<Vec<f32>>, output_sample_rate: u32) -> Self {
        Self {
            samples_rx,
            current: Vec::new(),
            offset: 0,
            output_sample_rate,
            source_position: 0.0,
            last_frame: [0.0, 0.0],
            fade_remaining_frames: 0,
            fade_total_frames: voice_output_underrun_fade_frames(output_sample_rate),
        }
    }

    pub(super) fn next_stereo_frame(&mut self) -> Option<[f32; 2]> {
        let frame = if self.output_sample_rate == DISCORD_VOICE_SAMPLE_RATE {
            self.next_native_stereo_frame()
        } else {
            self.next_resampled_stereo_frame()
        };
        match frame {
            Some(frame) => {
                self.last_frame = frame;
                self.fade_remaining_frames = self.fade_total_frames;
                Some(frame)
            }
            None => self.next_fade_stereo_frame(),
        }
    }

    fn next_native_stereo_frame(&mut self) -> Option<[f32; 2]> {
        loop {
            if self.offset + 1 < self.current.len() {
                let frame = [self.current[self.offset], self.current[self.offset + 1]];
                self.offset += usize::from(DISCORD_VOICE_CHANNELS);
                return Some(frame);
            }
            if !self.receive_next_samples() {
                return None;
            }
        }
    }

    fn next_resampled_stereo_frame(&mut self) -> Option<[f32; 2]> {
        loop {
            let frame_count = self.current.len() / usize::from(DISCORD_VOICE_CHANNELS);
            if frame_count == 0 || self.source_position >= frame_count as f64 {
                self.source_position -= frame_count as f64;
                if self.source_position < 0.0 {
                    self.source_position = 0.0;
                }
                if !self.receive_next_samples() {
                    return None;
                }
                continue;
            }

            let base_frame = self.source_position.floor() as usize;
            let next_frame = (base_frame + 1).min(frame_count - 1);
            let fraction = (self.source_position - base_frame as f64) as f32;
            let frame = interpolate_voice_stereo_frame(
                voice_stereo_frame_at(&self.current, base_frame),
                voice_stereo_frame_at(&self.current, next_frame),
                fraction,
            );
            self.source_position +=
                f64::from(DISCORD_VOICE_SAMPLE_RATE) / f64::from(self.output_sample_rate.max(1));
            return Some(frame);
        }
    }

    fn receive_next_samples(&mut self) -> bool {
        match self.samples_rx.try_recv() {
            Ok(samples) => {
                self.current = samples;
                self.offset = 0;
                true
            }
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => false,
        }
    }

    fn next_fade_stereo_frame(&mut self) -> Option<[f32; 2]> {
        if self.fade_remaining_frames == 0 || self.fade_total_frames == 0 {
            return None;
        }
        let gain = self.fade_remaining_frames as f32 / (self.fade_total_frames + 1) as f32;
        self.fade_remaining_frames -= 1;
        Some([self.last_frame[0] * gain, self.last_frame[1] * gain])
    }

    pub(super) fn clear_pending(&mut self) {
        self.current.clear();
        self.offset = 0;
        self.source_position = 0.0;
        self.last_frame = [0.0, 0.0];
        self.fade_remaining_frames = 0;
        while self.samples_rx.try_recv().is_ok() {}
    }
}

fn voice_output_underrun_fade_frames(output_sample_rate: u32) -> usize {
    ((output_sample_rate.max(1) * VOICE_OUTPUT_UNDERRUN_FADE_MILLIS) / 1_000).max(1) as usize
}

fn voice_stereo_frame_at(samples: &[f32], frame: usize) -> [f32; 2] {
    let offset = frame * usize::from(DISCORD_VOICE_CHANNELS);
    [samples[offset], samples[offset + 1]]
}

fn interpolate_voice_stereo_frame(left: [f32; 2], right: [f32; 2], fraction: f32) -> [f32; 2] {
    [
        left[0] + (right[0] - left[0]) * fraction,
        left[1] + (right[1] - left[1]) * fraction,
    ]
}
