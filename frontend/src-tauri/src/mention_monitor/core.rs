//! Platform-independent monitor policy. No Tauri, audio devices, or network calls.
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use unicode_normalization::UnicodeNormalization;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WatchKind {
    Name,
    Project,
    Topic,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WatchItem {
    pub id: String,
    pub label: String,
    pub kind: WatchKind,
    pub aliases: Vec<String>,
    #[serde(default)]
    pub description: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AiProvider {
    #[default]
    BuiltIn,
    Ollama,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct MonitorConfig {
    pub version: u32,
    pub items: Vec<WatchItem>,
    pub cooldown_seconds: u64,
    pub sound: bool,
    pub recap_enabled: bool,
    pub ollama_model: String,
    pub ai_provider: AiProvider,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            version: 1,
            items: vec![],
            cooldown_seconds: 30,
            sound: true,
            recap_enabled: true,
            ollama_model: String::new(),
            ai_provider: AiProvider::BuiltIn,
        }
    }
}

impl MonitorConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("Unsupported monitoring settings version".into());
        }
        if self.items.len() > 50 || !(5..=300).contains(&self.cooldown_seconds) {
            return Err(
                "Use at most 50 watch items and a cooldown between 5 and 300 seconds".into(),
            );
        }
        let mut ids = std::collections::HashSet::new();
        let topic_chars: usize = self
            .items
            .iter()
            .filter(|i| i.kind == WatchKind::Topic)
            .map(|i| i.description.chars().count() + i.label.chars().count())
            .sum();
        if topic_chars > 6000 {
            return Err("Keep the combined topic descriptions under 6000 characters for fast local analysis".into());
        }
        for item in &self.items {
            if item.id.is_empty()
                || item.id.len() > 80
                || !ids.insert(&item.id)
                || normalize(&item.label).is_empty()
                || item.label.len() > 160
                || item.description.len() > 1000
                || item.aliases.len() > 20
                || item
                    .aliases
                    .iter()
                    .any(|a| normalize(a).is_empty() || a.len() > 160)
            {
                return Err(
                    "Each watch item needs a unique ID, a short label, and nonempty aliases".into(),
                );
            }
            if item.kind == WatchKind::Topic && item.description.trim().is_empty() {
                return Err("Add a description of the work you want each topic to match".into());
            }
        }
        if self.ollama_model.len() > 160 || self.ollama_model.chars().any(char::is_control) {
            return Err("Invalid local model name".into());
        }
        Ok(())
    }
    pub fn ai_configured(&self) -> bool {
        self.ai_provider == AiProvider::BuiltIn || !self.ollama_model.trim().is_empty()
    }
    pub fn needs_ai(&self) -> bool {
        self.recap_enabled || self.items.iter().any(|i| i.kind == WatchKind::Topic)
    }
}

pub fn normalize(text: &str) -> String {
    text.nfkc()
        .flat_map(char::to_lowercase)
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn contains_phrase(text: &str, phrase: &str) -> bool {
    let phrase = normalize(phrase);
    !phrase.is_empty() && format!(" {} ", normalize(text)).contains(&format!(" {} ", phrase))
}

pub fn clip(text: &str, chars: usize) -> String {
    text.chars().take(chars).collect()
}

fn excerpt_for_phrase(text: &str, phrase: &str) -> String {
    let chars: Vec<_> = text.chars().collect();
    // Overlapping windows ensure a long segment still shows the matched phrase.
    for start in (0..chars.len()).step_by(600) {
        let end = (start + 1200).min(chars.len());
        let excerpt: String = chars[start..end].iter().collect();
        if contains_phrase(&excerpt, phrase) {
            return format!(
                "{}{}{}",
                if start > 0 { "…" } else { "" },
                excerpt,
                if end < chars.len() { "…" } else { "" }
            );
        }
    }
    clip(text, 1200)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Segment {
    pub sequence_id: u64,
    pub text: String,
    pub audio_start_time: f64,
    pub audio_end_time: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MentionAlert {
    pub id: u64,
    pub item_id: String,
    pub label: String,
    pub kind: WatchKind,
    pub excerpt: String,
    pub summary: Option<String>,
    pub audio_time: f64,
    pub created_at_ms: u64,
    pub is_test: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct MonitorSnapshot {
    pub config: MonitorConfig,
    pub recording: bool,
    pub enabled: bool,
    pub paused: bool,
    pub snoozed_until_ms: u64,
    pub status: String,
    pub ai_status: String,
    pub transcript_lag_seconds: f64,
    pub alert: Option<MentionAlert>,
    pub recent: Vec<Segment>,
}

#[derive(Default)]
pub struct Monitor {
    pub config: MonitorConfig,
    pub session: u64,
    pub generation: u64,
    pub recording: bool,
    pub local_transcription: bool,
    pub enabled: bool,
    pub paused: bool,
    pub snoozed_until_ms: u64,
    pub ai_status: String,
    pub delivery_error: Option<String>,
    pub transcript_lag_seconds: f64,
    pub alert: Option<MentionAlert>,
    pub minimum_audio_time: f64,
    recent: VecDeque<Segment>,
    last_alerts: HashMap<String, (u64, String)>,
    next_alert_id: u64,
}

impl Monitor {
    pub fn start_session(&mut self) -> u64 {
        self.session += 1;
        self.invalidate();
        self.recording = true;
        self.local_transcription = false;
        self.enabled = false;
        self.paused = false;
        self.snoozed_until_ms = 0;
        self.transcript_lag_seconds = 0.0;
        self.minimum_audio_time = 0.0;
        self.recent.clear();
        self.last_alerts.clear();
        self.session
    }
    pub fn invalidate(&mut self) {
        self.generation += 1;
        self.alert = None;
        self.delivery_error = None;
    }
    pub fn stop(&mut self) {
        self.invalidate();
        self.recording = false;
        self.enabled = false;
        self.recent.clear();
    }
    pub fn active(&self, now: u64) -> bool {
        self.recording
            && self.local_transcription
            && self.enabled
            && !self.paused
            && now >= self.snoozed_until_ms
    }
    pub fn accepts(&self, session: u64, generation: u64, now: u64) -> bool {
        self.session == session && self.generation == generation && self.active(now)
    }
    pub fn snapshot(&self, now: u64) -> MonitorSnapshot {
        let status = if !self.recording {
            "Start a recording to enable alerts"
        } else if !self.local_transcription {
            "Local transcription is required"
        } else if let Some(error) = &self.delivery_error {
            error
        } else if !self.enabled {
            "Alerts off for this meeting"
        } else if self.paused {
            "Monitoring paused with recording"
        } else if now < self.snoozed_until_ms {
            "Alerts snoozed"
        } else if self.transcript_lag_seconds > 5.0 {
            "Monitoring catching up — transcription is behind live audio"
        } else {
            "Monitoring active"
        };
        MonitorSnapshot {
            config: self.config.clone(),
            recording: self.recording,
            enabled: self.enabled,
            paused: self.paused,
            snoozed_until_ms: self.snoozed_until_ms,
            status: status.into(),
            ai_status: self.ai_status.clone(),
            transcript_lag_seconds: self.transcript_lag_seconds,
            alert: self.alert.clone(),
            recent: self.recent.iter().cloned().collect(),
        }
    }
    /// Returns only fresh explicit matches. Context and aliases span segment edges.
    pub fn observe(&mut self, session: u64, mut segment: Segment, now: u64) -> Vec<MentionAlert> {
        if session != self.session
            || !self.active(now)
            || segment.text.trim().is_empty()
            || !segment.audio_start_time.is_finite()
            || !segment.audio_end_time.is_finite()
            || segment.audio_end_time < segment.audio_start_time
            || segment.audio_end_time <= self.minimum_audio_time
        {
            return vec![];
        }
        segment.text = clip(&segment.text, 4000);
        if self
            .recent
            .iter()
            .any(|s| s.sequence_id == segment.sequence_id)
        {
            return vec![];
        }
        let previous = self
            .recent
            .back()
            .filter(|s| segment.audio_start_time - s.audio_end_time <= 3.0)
            .map(|s| s.text.clone())
            .unwrap_or_default();
        let old = format!(" {} ", normalize(&previous));
        let current = format!(" {} ", normalize(&segment.text));
        let combined = format!("{} {}", previous, segment.text);
        let combined_normalized = format!(" {} ", normalize(&combined));
        self.recent.push_back(segment.clone());
        while self.recent.len() > 40
            || self
                .recent
                .front()
                .map_or(false, |s| segment.audio_end_time - s.audio_end_time > 60.0)
        {
            self.recent.pop_front();
        }
        // Hard character budget as well as a time/segment budget.
        while self.recent.len() > 1
            && self.recent.iter().map(|s| s.text.len()).sum::<usize>() > 16000
        {
            self.recent.pop_front();
        }
        let mut items = self.config.items.clone();
        // A direct name mention wins when the same segment also names a project.
        items.sort_by_key(|i| if i.kind == WatchKind::Name { 1 } else { 0 });
        let mut alerts = vec![];
        for item in items.iter().filter(|i| i.kind != WatchKind::Topic) {
            let phrases = std::iter::once(&item.label).chain(item.aliases.iter());
            let matched = phrases.into_iter().find_map(|p| {
                let phrase = format!(" {} ", normalize(p));
                if current.contains(&phrase) {
                    Some((p, true))
                } else if combined_normalized.contains(&phrase) && !old.contains(&phrase) {
                    Some((p, false))
                } else {
                    None
                }
            });
            if let Some((phrase, in_current)) = matched {
                let excerpt =
                    excerpt_for_phrase(if in_current { &segment.text } else { &combined }, phrase);
                if let Some(alert) =
                    self.create_alert(item, excerpt, segment.audio_end_time, now, false)
                {
                    alerts.push(alert);
                }
            }
        }
        alerts
    }
    pub fn context(&self) -> String {
        self.recent
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }
    pub fn latest(&self) -> Option<&Segment> {
        self.recent.back()
    }
    pub fn create_alert(
        &mut self,
        item: &WatchItem,
        excerpt: String,
        audio_time: f64,
        now: u64,
        is_test: bool,
    ) -> Option<MentionAlert> {
        let newest = excerpt.lines().last().unwrap_or(&excerpt);
        let fingerprint = normalize(newest);
        if !is_test {
            if let Some((last, previous)) = self.last_alerts.get(&item.id) {
                // A distinct directed question can break the cooldown. Reprocessing cannot.
                let direct_question = item.kind == WatchKind::Name
                    && newest.contains('?')
                    && std::iter::once(&item.label)
                        .chain(item.aliases.iter())
                        .any(|p| contains_phrase(newest, p))
                    && previous != &fingerprint;
                if now.saturating_sub(*last) < self.config.cooldown_seconds * 1000
                    && !direct_question
                {
                    if let Some(alert) = self.alert.as_mut().filter(|a| a.item_id == item.id) {
                        alert.excerpt = excerpt;
                        alert.audio_time = audio_time;
                        alert.summary = None;
                    }
                    return None;
                }
            }
            self.last_alerts.insert(item.id.clone(), (now, fingerprint));
        }
        self.next_alert_id += 1;
        let alert = MentionAlert {
            id: self.next_alert_id,
            item_id: item.id.clone(),
            label: item.label.clone(),
            kind: item.kind.clone(),
            excerpt,
            summary: None,
            audio_time,
            created_at_ms: now,
            is_test,
        };
        self.alert = Some(alert.clone());
        Some(alert)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn monitor() -> Monitor {
        let mut m = Monitor::default();
        m.config.items = vec![WatchItem {
            id: "name".into(),
            label: "Alex Morgan".into(),
            kind: WatchKind::Name,
            aliases: vec!["Alex".into()],
            description: String::new(),
        }];
        m.start_session();
        m.local_transcription = true;
        m.enabled = true;
        m
    }
    fn segment(id: u64, text: &str) -> Segment {
        Segment {
            sequence_id: id,
            text: text.into(),
            audio_start_time: id as f64,
            audio_end_time: id as f64 + 1.0,
        }
    }
    #[test]
    fn unicode_and_word_boundaries() {
        assert!(contains_phrase("JOSÉ, review this", "Jose\u{301}"));
        assert!(!contains_phrase("Annette", "Ann"));
        assert!(contains_phrase("PAYMENT-retries", "payment retries"));
        assert!(!contains_phrase("hello", "  "));
    }
    #[test]
    fn split_alias_and_duplicate_delivery() {
        let mut m = monitor();
        m.config.items[0].aliases.clear();
        assert!(m
            .observe(m.session, segment(1, "Could Alex"), 1000)
            .is_empty());
        assert_eq!(
            m.observe(m.session, segment(2, "Morgan review this?"), 2000)
                .len(),
            1
        );
        assert!(m
            .observe(m.session, segment(2, "Morgan review this?"), 3000)
            .is_empty());
    }
    #[test]
    fn cooldown_and_fresh_question() {
        let mut m = monitor();
        assert_eq!(
            m.observe(m.session, segment(1, "Alex shipped it."), 1000)
                .len(),
            1
        );
        assert!(m
            .observe(m.session, segment(2, "Alex has the details."), 2000)
            .is_empty());
        assert_eq!(
            m.observe(m.session, segment(3, "Alex, can you confirm?"), 3000)
                .len(),
            1
        );
        assert!(m
            .observe(m.session, segment(4, "Alex, can you confirm?"), 4000)
            .is_empty());
        assert!(m
            .observe(m.session, segment(5, "Alex has the answer."), 5000)
            .is_empty());
    }
    #[test]
    fn session_pause_snooze_and_stop_reject_stale_work() {
        let mut m = monitor();
        let session = m.session;
        let generation = m.generation;
        m.paused = true;
        assert!(!m.accepts(session, generation, 1000));
        m.paused = false;
        m.snoozed_until_ms = 2000;
        assert!(!m.accepts(session, generation, 1999));
        assert!(m.accepts(session, generation, 2000));
        m.invalidate();
        assert!(!m.accepts(session, generation, 3000));
        m.stop();
        assert!(m.observe(session, segment(1, "Alex"), 5000).is_empty());
        m.start_session();
        m.enabled = true;
        m.local_transcription = true;
        assert!(m.observe(session, segment(1, "Alex"), 5000).is_empty());
    }
    #[test]
    fn config_rejects_empty_aliases_and_duplicate_ids() {
        let mut m = monitor();
        assert!(m.config.validate().is_ok());
        m.config.items[0].aliases.push("!!!".into());
        assert!(m.config.validate().is_err());
        m.config.items[0].aliases.clear();
        m.config.items.push(m.config.items[0].clone());
        assert!(m.config.validate().is_err());
    }
    #[test]
    fn context_is_bounded_and_disabled_means_no_capture() {
        let mut m = monitor();
        m.enabled = false;
        m.observe(m.session, segment(1, "private"), 1);
        assert!(m.context().is_empty());
        m.enabled = true;
        for id in 1..=200 {
            m.observe(m.session, segment(id, &"x".repeat(4000)), id * 1000);
        }
        assert!(m.context().len() < 17000);
    }
    #[test]
    fn excerpt_contains_the_trigger_even_in_a_long_segment() {
        let mut m = monitor();
        let long = format!(
            "{} Alex should review the release.",
            "unrelated ".repeat(350)
        );
        let alerts = m.observe(m.session, segment(1, &long), 1000);
        assert_eq!(alerts.len(), 1);
        assert!(alerts[0].excerpt.contains("Alex"));
        assert!(alerts[0].excerpt.chars().count() <= 1202);
    }
    #[test]
    fn queued_audio_from_before_arming_or_snooze_expiry_does_not_alert() {
        let mut m = monitor();
        m.minimum_audio_time = 10.0;
        assert!(m
            .observe(m.session, segment(1, "Alex, review this?"), 1000)
            .is_empty());
        assert_eq!(
            m.observe(m.session, segment(11, "Alex, review this?"), 2000)
                .len(),
            1
        );
    }
}
