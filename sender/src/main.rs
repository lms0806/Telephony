use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use tokio::net::UdpSocket;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .expect("No input device available");

    println!("Input device: {}", device.name()?);

    let supported_config = device.default_input_config()?;
    println!("Default config: {:?}", supported_config);

    let sample_format = supported_config.sample_format();
    let config: cpal::StreamConfig = supported_config.into();

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let target: SocketAddr = "127.0.0.1:5000".parse().unwrap();

    let stream = match sample_format {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config,
            move |data: &[f32], _| {
                let mut prev = 0.0;

                let samples: Vec<i16> = data
                    .iter()
                    .map(|&x| {
                        // 🔥 Low-pass filter
                        let filtered = prev * 0.8 + x * 0.2;
                        prev = filtered;

                        // 🔥 부드러운 노이즈 게이트
                        let gated = if filtered.abs() < 0.01 {
                            filtered * 0.2
                        } else {
                            filtered
                        };

                        // 🔥 soft clipping + gain 조정
                        let processed = (gated * 0.9).tanh();

                        (processed * i16::MAX as f32) as i16
                    })
                    .collect();

                let bytes = bytemuck::cast_slice(&samples);

                // 🔥 20ms 패킷 (48000Hz 기준)
                for chunk in bytes.chunks(960 * 2) {
                    let _ = socket.try_send_to(chunk, target);
                }
            },
            |err| eprintln!("Stream error: {}", err),
            None,
        )?,

        cpal::SampleFormat::I16 => device.build_input_stream(
            &config,
            move |data: &[i16], _| {
                let mut prev = 0.0;

                let samples: Vec<i16> = data
                    .iter()
                    .map(|&x| {
                        let x = x as f32 / i16::MAX as f32;

                        // 🔥 Low-pass filter
                        let filtered = prev * 0.8 + x * 0.2;
                        prev = filtered;

                        // 🔥 Noise gate
                        if filtered.abs() < 0.02 {
                            return 0;
                        }

                        let processed = filtered.tanh();

                        (processed * i16::MAX as f32) as i16
                    })
                    .collect();

                let bytes = bytemuck::cast_slice(&samples);

                for chunk in bytes.chunks(960 * 2) {
                    let _ = socket.try_send_to(chunk, target);
                }
            },
            |err| eprintln!("Stream error: {}", err),
            None,
        )?,

        cpal::SampleFormat::U16 => device.build_input_stream(
            &config,
            move |data: &[u16], _| {
                let mut prev = 0.0;

                let samples: Vec<i16> = data
                    .iter()
                    .map(|&x| {
                        let x = (x as f32 / u16::MAX as f32) * 2.0 - 1.0;

                        let filtered = prev * 0.8 + x * 0.2;
                        prev = filtered;

                        if filtered.abs() < 0.02 {
                            return 0;
                        }

                        let processed = filtered.tanh();

                        (processed * i16::MAX as f32) as i16
                    })
                    .collect();

                let bytes = bytemuck::cast_slice(&samples);

                for chunk in bytes.chunks(960 * 2) {
                    let _ = socket.try_send_to(chunk, target);
                }
            },
            |err| eprintln!("Stream error: {}", err),
            None,
        )?,

        _ => unreachable!(),
    };

    stream.play()?;

    println!("🎤 Noise-reduced audio streaming...");

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}