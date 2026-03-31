# 📞 Telephony

Rust 기반의 실시간 음성 통신(Voice Chat) 시스템입니다.  
마이크 입력을 캡처하여 네트워크를 통해 전송하고, 수신된 오디오 데이터를 재생하는 **간단한 VoIP(Voice over IP)** 구조를 구현합니다.

---

## 🏗️ Architecture

```
[Microphone]
     ↓
[CPAL Input Stream]
     ↓
[Audio Buffer]
     ↓
[UDP Sender (Tokio)]
     ↓
======================== Network ========================
     ↓
[UDP Receiver (Tokio)]
     ↓
[Audio Buffer]
     ↓
[CPAL Output Stream]
     ↓
[Speaker]
```

---

## 📦 Installation

```bash
git clone https://github.com/lms0806/Telephony.git
cd Telephony
```

---

## ▶️ Usage

### 1. 서버(수신 측) 실행

```bash
cd receiver
cargo run
```

### 2. 클라이언트(송신 측) 실행

```bash
cd sender
cargo run --bin sender
```

---

## 📂 Project Structure

```
Telephony/
├── sender/
│   └── src/
│       └── main.rs   # 오디오 캡처 및 전송
│
├── receiver/
│   └── src/
│       └── main.rs   # 오디오 수신 및 재생
│
└── README.md
```

---

## ⚡ Key Concepts

### 1. Audio Capture
`cpal`을 사용하여 마이크 입력을 실시간으로 수집합니다.

### 2. Packetization
오디오 데이터를 일정 크기의 버퍼로 나누어 UDP 패킷으로 전송합니다.

### 3. Networking
- UDP 기반 → 빠르지만 패킷 손실 가능
- TCP보다 낮은 지연(latency)

### 4. Playback
수신된 데이터를 다시 오디오 스트림으로 변환하여 재생합니다.
