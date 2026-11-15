#[path = "../../src/mention_monitor/core.rs"]
pub mod core;
pub mod mention_monitor {
    pub use crate::core;
}
#[path = "../../src/audio/live_speech_windows.rs"]
pub mod live_speech_windows;
#[path = "../../src/mention_monitor/local_ai.rs"]
pub mod local_ai;
