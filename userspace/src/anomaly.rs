use common::AnomalyEvent;
use log::{warn, info};
use crate::output::emit_anomaly;
use std::collections::{HashMap, VecDeque};
use std::time::{Instant, Duration};

pub struct AnomalyEngine {
    /// Maps PID to a sliding window of timestamps when anomalies occurred
    pid_history: HashMap<u32, VecDeque<Instant>>,
}

impl AnomalyEngine {
    pub fn new() -> Self {
        Self {
            pid_history: HashMap::new(),
        }
    }

    pub fn process_event(&mut self, event: &AnomalyEvent) {
        let now = Instant::now();
        let history = self.pid_history.entry(event.pid).or_insert_with(VecDeque::new);
        history.push_back(now);
        
        // Sliding window: Evict events older than 60 seconds
        let sixty_seconds_ago = now - Duration::from_secs(60);
        while let Some(&oldest) = history.front() {
            if oldest < sixty_seconds_ago {
                history.pop_front();
            } else {
                break;
            }
        }
        
        let anomaly_count_last_minute = history.len();
        
        // High-level correlation: If a process triggers > 5 anomalies a minute, escalate
        if anomaly_count_last_minute > 5 {
            warn!("CRITICAL: PID {} has triggered {} anomalies in the last 60 seconds! Potential exfiltration or attack.", event.pid, anomaly_count_last_minute);
        } else {
            info!("Anomaly detected for PID {}. (Total last min: {})", event.pid, anomaly_count_last_minute);
        }

        // Export event
        emit_anomaly(event);
    }
}
