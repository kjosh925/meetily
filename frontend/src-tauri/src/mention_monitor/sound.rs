//! App-owned chime: independent of notification permissions and OS sound names.
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::time::Duration;

fn sample(frame: usize, rate: u32) -> f32 {
    let t = frame as f32 / rate as f32 - 0.1;
    if !(0.0..0.7).contains(&t) {
        return 0.0;
    }
    let (t, frequency) = if t < 0.35 {
        (t, 740.0)
    } else {
        (t - 0.35, 988.0)
    };
    let envelope = (t / 0.015).min(1.0) * ((0.35 - t) / 0.12).min(1.0);
    0.24 * envelope * (std::f32::consts::TAU * frequency * t).sin()
}

/// Run on a dedicated thread: CPAL streams need to live until output is drained.
pub fn play() -> Result<(), String> {
    let device = cpal::default_host()
        .default_output_device()
        .ok_or("No audio output device")?;
    let supported = device.default_output_config().map_err(|e| e.to_string())?;
    let format = supported.sample_format();
    let config: cpal::StreamConfig = supported.into();
    let rate = config.sample_rate.0;
    let channels = config.channels as usize;
    let (done, receive) = std::sync::mpsc::sync_channel(2);
    macro_rules! build {
        ($ty:ty) => {{
            let mut frame = 0;
            let mut sent = false;
            let errors = done.clone();
            device.build_output_stream(
                &config,
                move |data: &mut [$ty], _: &cpal::OutputCallbackInfo| {
                    for output in data.chunks_mut(channels) {
                        let value = sample(frame, rate);
                        for channel in output {
                            *channel = <$ty as cpal::FromSample<f32>>::from_sample_(value);
                        }
                        frame += 1;
                    }
                    if !sent && frame >= rate as usize {
                        sent = true;
                        let _ = done.try_send(Ok(()));
                    }
                },
                move |error| {
                    let _ = errors.try_send(Err(error.to_string()));
                },
                None,
            )
        }};
    }
    let stream = match format {
        cpal::SampleFormat::I8 => build!(i8),
        cpal::SampleFormat::I16 => build!(i16),
        cpal::SampleFormat::I32 => build!(i32),
        cpal::SampleFormat::I64 => build!(i64),
        cpal::SampleFormat::U8 => build!(u8),
        cpal::SampleFormat::U16 => build!(u16),
        cpal::SampleFormat::U32 => build!(u32),
        cpal::SampleFormat::U64 => build!(u64),
        cpal::SampleFormat::F32 => build!(f32),
        cpal::SampleFormat::F64 => build!(f64),
        _ => return Err(format!("Unsupported output format: {format:?}")),
    }
    .map_err(|e| e.to_string())?;
    stream.play().map_err(|e| e.to_string())?;
    receive
        .recv_timeout(Duration::from_secs(4))
        .map_err(|_| "Audio output did not play the chime".to_string())??;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chime_is_audible_bounded_and_finishes_in_silence() {
        for rate in [16000, 44100, 48000] {
            let samples: Vec<_> = (0..rate as usize).map(|n| sample(n, rate)).collect();
            let rms = (samples.iter().map(|x| x * x).sum::<f32>() / rate as f32).sqrt();
            assert!(rms > 0.08, "silent or too quiet: {rms}");
            assert!(samples.iter().all(|s| s.abs() <= 0.25));
            assert!(samples[(rate as f32 * 0.81) as usize..]
                .iter()
                .all(|s| *s == 0.0));
        }
    }
    #[test]
    #[ignore = "Plays audible audio through the system default output"]
    fn real_output_plays_chime() {
        play().unwrap();
    }
}
