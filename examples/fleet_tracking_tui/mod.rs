use std::collections::{BTreeMap, VecDeque};
use std::io::{self, Write};

const MAX_MESSAGES: usize = 10;

#[derive(Debug)]
struct Vehicle {
    latitude: f64,
    longitude: f64,
}

pub struct Tui {
    vehicles: BTreeMap<String, Vehicle>,
    messages: VecDeque<String>,
}

impl Tui {
    pub fn new() -> Self {
        Self {
            vehicles: BTreeMap::new(),
            messages: VecDeque::new(),
        }
    }

    pub fn update_vehicle(&mut self, vehicle_id: impl Into<String>, latitude: f64, longitude: f64) {
        self.vehicles.insert(
            vehicle_id.into(),
            Vehicle {
                latitude,
                longitude,
            },
        );
        self.draw();
    }

    pub fn remove_vehicle(&mut self, vehicle_id: &str) {
        self.vehicles.remove(vehicle_id);
        self.draw();
    }

    pub fn log(&mut self, message: impl Into<String>) {
        if self.messages.len() == MAX_MESSAGES {
            self.messages.pop_front();
        }
        self.messages.push_back(message.into());
        self.draw();
    }

    fn draw(&self) {
        let vehicle_rows: Vec<_> = self
            .vehicles
            .iter()
            .map(|(vehicle_id, vehicle)| {
                (
                    vehicle_id,
                    vehicle.latitude.to_string(),
                    vehicle.longitude.to_string(),
                )
            })
            .collect();
        let vehicle_id_width = vehicle_rows
            .iter()
            .map(|(vehicle_id, _, _)| vehicle_id.len())
            .max()
            .unwrap_or("vehicle_id".len())
            .max("vehicle_id".len());
        let latitude_width = vehicle_rows
            .iter()
            .map(|(_, latitude, _)| latitude.len())
            .max()
            .unwrap_or("latitude".len())
            .max("latitude".len());
        let longitude_width = vehicle_rows
            .iter()
            .map(|(_, _, longitude)| longitude.len())
            .max()
            .unwrap_or("longitude".len())
            .max("longitude".len());

        let mut output = io::stdout().lock();
        write!(output, "\x1b[2J\x1b[H").expect("failed to clear the TUI");
        writeln!(
            output,
            "| {:<vehicle_id_width$} | {:<latitude_width$} | {:<longitude_width$} |",
            "vehicle_id", "latitude", "longitude"
        )
        .expect("failed to draw the TUI");
        for (vehicle_id, latitude, longitude) in vehicle_rows {
            //vehicle_id
            // 287c7db5c797c9c52819081a272fe01e
            writeln!(
                output,
                "| {:<vehicle_id_width$} | {:<latitude_width$} | {:<longitude_width$} |",
                vehicle_id, latitude, longitude
            )
            .expect("failed to draw the TUI");
        }
        for message in &self.messages {
            writeln!(output, "{message}").expect("failed to draw the TUI");
        }
        output.flush().expect("failed to flush the TUI");
    }
}
