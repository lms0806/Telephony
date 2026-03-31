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

    // 🔥 mono로 강제 (가능하면)
    //config.channels = 1;

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let target: SocketAddr = "127.0.0.1:5000".parse().unwrap();

    let stream = match sample_format {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config,
            move |data: &[f32], _| {
                let samples: Vec<i16> = data
                    .iter()
                    .map(|&x| {
                        let amplified = x * 1.2; // 🔥 gain (볼륨 증가)
                        (amplified.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
                    })
                    .collect();

                let bytes = bytemuck::cast_slice(&samples);

                for chunk in bytes.chunks(2048) { // 🔥 더 큰 패킷
                    let _ = socket.try_send_to(chunk, target);
                }
            },
            |err| eprintln!("Stream error: {}", err),
            None,
        )?,

        cpal::SampleFormat::I16 => device.build_input_stream(
            &config,
            move |data: &[i16], _| {
                let samples: Vec<i16> = data
                    .iter()
                    .map(|&x| ((x as f32) * 1.2).clamp(i16::MIN as f32, i16::MAX as f32) as i16)
                    .collect();

                let bytes = bytemuck::cast_slice(&samples);

                for chunk in bytes.chunks(2048) {
                    let _ = socket.try_send_to(chunk, target);
                }
            },
            |err| eprintln!("Stream error: {}", err),
            None,
        )?,

        cpal::SampleFormat::U16 => device.build_input_stream(
            &config,
            move |data: &[u16], _| {
                let samples: Vec<i16> = data
                    .iter()
                    .map(|&x| {
                        let normalized = x as f32 / u16::MAX as f32;
                        ((normalized * 2.0 - 1.0) * i16::MAX as f32) as i16
                    })
                    .collect();

                let bytes = bytemuck::cast_slice(&samples);

                for chunk in bytes.chunks(2048) {
                    let _ = socket.try_send_to(chunk, target);
                }
            },
            |err| eprintln!("Stream error: {}", err),
            None,
        )?,

        _ => unreachable!(),
    };

    stream.play()?;

    println!("🎤 Improved audio streaming...");

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}