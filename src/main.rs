use std::{
    process::exit,
    sync::{Arc, Mutex},
    thread::sleep, time::Duration,
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait}, Device, Host, Stream, StreamConfig
};


use rodio::{buffer, source::{self, Buffered, SineWave}, Decoder, OutputStream, Sink};
use rustfft::{algorithm::Radix4, num_complex::Complex, num_traits::Zero, Fft};

//const THRESHOLD: f32 = 0f32;

struct LocalDevice {
    device: Device,
    host: Host
}

impl LocalDevice {
    pub fn new(device: Device, host: Host) -> LocalDevice {
        LocalDevice { device, host }
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
            Radix4::new(2, rustfft::FftDirection::Forward);

        // 创建音频输入流
        let stream: Stream = self
            .device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // 将音频数据转换为复数
                    //tracing::info!("{:?}", data);
                    let mut complex: Vec<Complex<f32>> = data
                        .iter()
                        .map(|&sample| Complex::new(sample, 0.0))
                        .collect();
                    //tracing::info!("{:?}", complex);
                    // 执行FFT
                    fft.process(&mut complex);
                    

                    /*  对频谱进行处理以降噪，通过设置阈值
                    for freq in complex.iter_mut() {
                        if freq.norm() < THRESHOLD {
                            *freq = Complex::zero();
                        }
                    }
                    */

                    // 执行逆FFT
                    let ifft: Radix4<f32> = Radix4::new(2, rustfft::FftDirection::Inverse);
                    ifft.process(&mut complex);
                    
                    tracing::info!("Processed audio data");
                            // 将处理后的音频数据写入输出流
                            let real: Vec<f32> = complex.iter().map(|c| c.re).collect();
                            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                        
                            // 创建一个 Buffer 音频源
                            let audio_source: buffer::SamplesBuffer<f32> = buffer::SamplesBuffer::new(2, 48000, real.as_slice());
                        
                            // 创建一个 Sink 来控制音频播放
                            let sink: Sink = Sink::try_new(&stream_handle).unwrap();
                        
                            // 将 Buffer 音频源添加到 Sink 中
                            sink.append(audio_source);
                            sink.sleep_until_end();
                },
                move |err: cpal::StreamError| {
                    // 在这里处理错误
                    tracing::error!("An error occurred on stream: {:#?}", err);
                },
            )
            .unwrap();
        stream.play().unwrap();
        loop {}
    }
}

fn main() {
    let _g_logger: clia_tracing_config::WorkerGuard = clia_tracing_config::build()
        .directory("./dft/logs/")
        .format("full")
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
    tracing::info!("Start to Listen Std-SID");
    device().dft_and_make_art();
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
    LocalDevice::new(device_raw, host)
}