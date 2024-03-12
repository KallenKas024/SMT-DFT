# SMT-DFT
Dr.ShiMiTe is available now, go and buy it  now!
# 项目目标
实现一个能够实时(25ms间隔)获取麦克风输入并识别出所弹奏的和弦成分（由哪几个音组成 使用midi音值）的算法原型
# 开发语言
python
# 方案（暂定）
1. 使用pyaudio库获取麦克风输入
2. 提取一帧音频信号并降噪处理
3. 使用dft（离散傅里叶变换）转换成频率成分图
4. 设置一响度下限并将低于此值的部分设为0
5. 取几个峰值对应频率作为此刻的和弦成分