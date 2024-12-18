use crate::exporter::Exporter;
use crate::model::Record;
use clap::Parser;
use futures_util::lock::MutexGuard;
use prometheus_exporter::prometheus::register_counter_vec;
use std::collections::HashMap;
use tracing::{debug, info, trace, warn};

use prometheus_exporter::prometheus::core::{AtomicF64, GenericCounter};
use prometheus_exporter::{self, prometheus::register_counter};

#[derive(Parser)]
pub struct PrometheusOutput {
    // Port of prometheus /metrics listener
    #[clap()]
    port: String,
}

impl PrometheusOutput {
    pub fn exporter(&self) -> Box<dyn Exporter> {
        let bind_addr = format!("0.0.0.0:{}", self.port);
        info!("setting up exporter {:?}", bind_addr);
        let binding = bind_addr.parse().unwrap();
        let exporter = prometheus_exporter::start(binding).unwrap();

        // 2024-12-18T20:28:15.162874Z  INFO docker_activity::exporter::prometheus: handling record in exporter Record
        // { container_id: "c81cdc3ea969e03ccef28c8cd4c37dccb4bc0fe070c2e5b26a4e4a8d9a8fe6fd", container_name: "transmission",
        // ts: 1734553695, pid_count: Some(13), pid_limit: Some(18446744073709551615), memory_usage: Some(2920574976), memory_limit: Some(67207430144),
        // cpu_percent: 6.676557863501484e-6, cpu_count: 20, cpu_energy: Some(622.0923115727003) }
        let mut counters: HashMap<String, GenericCounter<AtomicF64>> = HashMap::new();

        Box::new(PrometheusExporter { exporter, counters })
    }
}

pub struct PrometheusExporter {
    // Exporter of metrics
    exporter: prometheus_exporter::Exporter,
    // counter
    counters: HashMap<String, GenericCounter<AtomicF64>>,
}

impl Exporter for PrometheusExporter {
    fn handle(&mut self, record: Record) -> Result<(), String> {
        let name = record.container_name.clone();
        info!(
            "handling record in exporter for {:?}, val: {:?}",
            name, record.cpu_energy
        );
        if record.cpu_energy == None {
            return Ok(());
        }

        if !self.counters.contains_key(&name) {
            let metric_name = format!(
                "docker_activity_cpu_power_{}",
                record.container_name.replace("-", "_")
            );
            info!("metric name: {:?}", metric_name);
            let counter =
                register_counter!(metric_name, "docker activity cpu power for a container")
                    .unwrap();
            self.counters.insert(name.clone(), counter);
        }

        let counter = self.counters.get(&name);
        if let Some(counter) = counter {
            counter.inc_by(record.cpu_energy.unwrap());
        }
        Ok(())
    }
}
