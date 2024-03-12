import pyaudio
import numpy as np

# 配置音频输入参数
CHUNK = 1024
FORMAT = pyaudio.paInt16
CHANNELS = 1
RATE = 44100

# 创建音频输入流
audio = pyaudio.PyAudio()
stream = audio.open(format=FORMAT, channels=CHANNELS,
                    rate=RATE, input=True, frames_per_buffer=CHUNK)

def calculate_db(audio_data):
    # 计算分贝数
    rms = np.sqrt(np.mean(np.square(audio_data)))
    db = 20 * np.log10(rms)
    return db

# 启动音频流
stream.start_stream()

print("Listening for audio...")

while True:
    try:
        # 读取音频数据
        audio_data = np.frombuffer(stream.read(CHUNK), dtype=np.int16)
        db = calculate_db(audio_data)
        print(f"Current dB level: {db} dB")
    except KeyboardInterrupt:
        break

# 停止音频流
stream.stop_stream()
stream.close()
audio.terminate()
