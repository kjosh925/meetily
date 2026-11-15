//! Sample cursor for capped live speech. Keeps a short overlap for boundary words.
#[derive(Default)]
pub struct LiveSpeechWindows {
    pub emitted: usize,
}

impl LiveSpeechWindows {
    pub fn ready(&mut self, samples: usize) -> Option<std::ops::Range<usize>> {
        const WINDOW: usize = 3 * 16000;
        const OVERLAP: usize = 4800;
        if samples.saturating_sub(self.emitted) < WINDOW {
            return None;
        }
        let range = self.emitted..self.emitted + WINDOW;
        self.emitted = range.end - OVERLAP;
        Some(range)
    }
    pub fn tail(&self, samples: usize) -> std::ops::Range<usize> {
        self.emitted.min(samples)..samples
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn continuous_speech_is_bounded_and_boundary_audio_is_preserved() {
        let mut windows = LiveSpeechWindows::default();
        assert!(windows.ready(47999).is_none());
        assert_eq!(windows.ready(48000), Some(0..48000));
        assert_eq!(windows.ready(91200), Some(43200..91200));
        assert_eq!(windows.tail(100000), 86400..100000);
        assert_eq!(windows.tail(100), 100..100);
    }
}
