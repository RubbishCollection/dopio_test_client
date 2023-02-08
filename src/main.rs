use sysinfo::{CpuExt, System, SystemExt};

use reqwest::Client;

use std::time::Duration;

use serde::Serialize;

#[derive(Serialize)]
struct NewChart {
    caption: String,
    #[serde(rename = "type")]
    chart_type: String,
    y_start: f64,
    y_end: f64,
    interval: u64,
    index: usize,
    tti: u64,
    viewport_size: usize,
}

#[derive(Serialize)]
struct Data {
    label: String,
    value: f64,
}

#[derive(Serialize)]
struct Label {
    name: String,
    r: u8,
    g: u8,
    b: u8,
}

struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    fn make_label(&self, name: String) -> Label {
        Label {
            name,
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}

#[tokio::main]
async fn main() {
    let mut sys = System::new_all();

    let client = Client::new();

    let interval = Duration::from_millis(500);

    let index = 1;

    let colors = vec![Color::new(255, 0, 0), Color::new(0, 255, 0), Color::new(0, 0, 255)
    ,Color::new(255, 255, 0), Color::new(0, 255, 255), Color::new(255, 0, 255),
    Color::new(127, 0, 0), Color::new(0, 127, 0), Color::new(0, 0, 127),
    Color::new(127, 127, 0), Color::new(0, 127, 127), Color::new(127, 0, 127)];

    let new_chart = NewChart {
        caption: "Cpu Usage".to_string(),
        chart_type: "pass-thru".to_string(),
        interval: interval.as_millis() as u64,
        y_start: 0.,
        y_end: 100.,
        index,
        tti: 60000,
        viewport_size: 50,
    };

    client
        .get("http://127.0.0.1:3000/new_chart")
        .query(&new_chart)
        .send()
        .await
        .unwrap();

    let new_label = Label {
        name: "Average".to_string(),
        r: 0,
        g: 0,
        b: 0,
    };

    client
        .get(format!("http://127.0.0.1:3000/new_label/{}", index))
        .query(&new_label)
        .send()
        .await
        .unwrap();

    let len = sys.cpus().len() as u8;

    for (i, _) in sys.cpus().iter().enumerate() {
        let new_label = colors[i].make_label(format!("Cpu{}", i));

        client
            .get(format!("http://127.0.0.1:3000/new_label/{}", index))
            .query(&new_label)
            .send()
            .await
            .unwrap();
    }

    loop {
        let mut sum = 0.;

        sys.refresh_cpu();

        let len = sys.cpus().len();

        for (i, cpu) in sys.cpus().iter().enumerate() {
            sum += cpu.cpu_usage() / len as f32;

            client
            .get(format!("http://127.0.0.1:3000/insert/{}", index))
            .query(&Data {
                label: format!("Cpu{}", i),
                value: cpu.cpu_usage() as f64,
            })
            .send()
            .await
            .unwrap();
        }

        println!("Cpu load: {:#?}", sum);

        client
            .get(format!("http://127.0.0.1:3000/insert/{}", index))
            .query(&Data {
                label: "Average".to_string(),
                value: sum as f64,
            })
            .send()
            .await
            .unwrap();

        std::thread::sleep(interval);
    }
}
