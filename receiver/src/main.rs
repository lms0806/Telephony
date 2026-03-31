use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use tokio::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:5000").await?;

    // 🔥 i16 + VecDeque (FIFO)
    let buffer = Arc::new(Mutex::new(VecDeque::<i16>::new()));
    let buf_clone = buffer.clone();

    // 네트워크 수신
    tokio::spawn(async move {
        let mut recv_buf = [0u8; 4096];
        loop {
            let (len, _) = socket.recv_from(&mut recv_buf).await.unwrap();

            // 🔥 i16로 변환
            let samples: &[i16] = bytemuck::cast_slice(&recv_buf[..len]);

            let mut buf = buf_clone.lock().unwrap();
            for &s in samples {
                buf.push_back(s);
            }
        }
    });

    // 오디오 출력
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();

    let supported_config = device.default_output_config()?;
    let sample_format = supported_config.sample_format();
    let config: cpal::StreamConfig = supported_config.into();

    let stream = match sample_format {
        cpal::SampleFormat::F32 => device.build_output_stream(
            &config,
            move |output: &mut [f32], _| {
                let mut buf = buffer.lock().unwrap();

                for sample in output.iter_mut() {
                    if let Some(v) = buf.pop_front() {
                        *sample = v as f32 / i16::MAX as f32;
                    } else {
                        *sample = 0.0;
                    }
                }
            },
            |err| eprintln!("error: {}", err),
            None,
        )?,

        cpal::SampleFormat::I16 => device.build_output_stream(
            &config,
            move |output: &mut [i16], _| {
                let mut buf = buffer.lock().unwrap();

                for sample in output.iter_mut() {
                    *sample = buf.pop_front().unwrap_or(0);
                }
            },
            |err| eprintln!("error: {}", err),
            None,
        )?,

        cpal::SampleFormat::U16 => device.build_output_stream(
            &config,
            move |output: &mut [u16], _| {
                let mut buf = buffer.lock().unwrap();

                for sample in output.iter_mut() {
                    if let Some(v) = buf.pop_front() {
                        *sample = ((v as f32 / i16::MAX as f32 + 1.0) * 0.5 * u16::MAX as f32) as u16;
                    } else {
                        *sample = 0;
                    }
                }
            },
            |err| eprintln!("error: {}", err),
            None,
        )?,

        _ => unreachable!(),
    };

    stream.play()?;

    println!("🔊 Receiving audio...");

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}