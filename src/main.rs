use std::{
    process::exit,
    sync::{Arc, Mutex},
    thread,
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Host, Stream, StreamConfig,
};
use plotters::{
    backend::BitMapBackend,
    chart::ChartBuilder,
    drawing::IntoDrawingArea,
    element::Circle,
    style::{Color, IntoFont as _, RED, WHITE},
};
use rustfft::{algorithm::Radix4, num_complex::Complex, num_traits::Zero, Fft};

const THRESHOLD: f32 = 0.1f32;

struct LocalDevice {
    device: Device,
}

impl LocalDevice {
    pub fn new(device: Device) -> LocalDevice {
        LocalDevice { device }
    }

    pub fn get_raw(self) -> Device {
        self.device
    }

    pub fn get_name(self) -> String {
        let device_raw_opt: Result<String, cpal::DeviceNameError> = self.device.name();
        match device_raw_opt {
            Ok(v) => v,
            Err(_e) => {
                tracing::error!("No Input device available.");
                exit(1);
            }
        }
    }

    pub fn dft_and_make_art(self) {
        let config: StreamConfig = match self.device.default_input_config() {
            Ok(v) => v.into(),
            Err(_e) => {
                tracing::error!("No Default Input Config available.");
                exit(1);
            }
        };
        // 创建FFT实例
        let fft: Radix4<f32> =
            Radix4::new(config.channels as usize, rustfft::FftDirection::Forward);
        let processed_data: Arc<Mutex<Vec<Complex<f32>>>> =
            Arc::new(Mutex::new(Vec::<Complex<f32>>::new()));
        let processed_data_clone: Arc<Mutex<Vec<Complex<f32>>>> = Arc::clone(&processed_data);
        // 创建音频输入流
        let _stream: Stream = self
            .device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // 将音频数据转换为复数
                    let mut complex: Vec<Complex<f32>> = data
                        .iter()
                        .map(|&sample| Complex::new(sample, 0.0))
                        .collect();

                    // 执行FFT
                    fft.process(&mut complex);

                    // 对频谱进行处理以降噪，通过设置阈值
                    for freq in complex.iter_mut() {
                        if freq.norm() < THRESHOLD {
                            *freq = Complex::zero();
                        }
                    }

                    // 执行逆FFT
                    fft.process(&mut complex);
                    tracing::error!("Processed audio data: {:?}", &complex);
                    let mut processed_data: std::sync::MutexGuard<'_, Vec<Complex<f32>>> =
                        processed_data.lock().unwrap();
                    *processed_data = complex;
                },
                move |err: cpal::StreamError| {
                    // 在这里处理错误
                    tracing::error!("An error occurred on stream: {}", err);
                },
            )
            .unwrap();

        thread::spawn(move || {
            loop {
                let processed_data: std::sync::MutexGuard<'_, Vec<Complex<f32>>> =
                    processed_data_clone.lock().unwrap();
                // 在这里访问处理后的数据
                println!("Processed audio data: {:?}", *processed_data);

                // 创建一个新的图表
                let root = BitMapBackend::new("plot.png", (640, 480)).into_drawing_area();
                root.fill(&WHITE).unwrap();
                let mut chart = ChartBuilder::on(&root)
                    .caption("Frequency Spectrum", ("Arial", 50).into_font())
                    .margin(5)
                    .x_label_area_size(30)
                    .y_label_area_size(30)
                    .build_ranged(0f32..1f32, 0f32..1f32)
                    .unwrap();

                // 绘制频率成分图
                chart.configure_mesh().draw().unwrap();
                chart
                    .draw_series(processed_data.iter().enumerate().map(|(i, freq)| {
                        let x = i as f32 / processed_data.len() as f32;
                        let y = freq.norm();
                        Circle::new((x, y), 1, RED.filled())
                    }))
                    .unwrap();
            }
        });
    }
}

fn main() {
    let _g_logger: clia_tracing_config::WorkerGuard = clia_tracing_config::build()
        .directory("./dft/logs/")
        .filter_level("info")
        .with_ansi(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_source_location(true)
        .with_target(true)
        .file_name("lastrun.log")
        .rolling("daily")
        .to_stdout(true)
        .init();

    tracing::info!("Start DFT-System");
    let device_local: LocalDevice = device();
    tracing::info!("Get the SoundInputDevice --> {:?}", device_local.get_name());
}

fn get_host() -> Host {
    cpal::default_host()
}

fn device() -> LocalDevice {
    let host: Host = get_host();
    let device_raw_opt: Option<Device> = host.default_input_device();
    let device_raw: Device = match device_raw_opt {
        Some(v) => v,
        None => {
            tracing::error!("No Input device available.");
            exit(1);
        }
    };
    LocalDevice::new(device_raw)
}
