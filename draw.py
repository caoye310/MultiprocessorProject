import pandas as pd
import matplotlib.pyplot as plt

# 读取 CSV 文件
file_path = "memory_data.csv"
data = pd.read_csv(file_path, header=None, names=["Time (s)", "Available Memory (GB)"])

# 绘制折线图
plt.figure(figsize=(10, 6))
plt.plot(data["Time (s)"]/10, data["Available Memory (GB)"], marker=None, linestyle='-', color='b')

# 添加图表标题和标签
plt.title("System Available Memory Over Time", fontsize=16)
plt.xlabel("Time (seconds)", fontsize=14)
plt.ylabel("Available Memory (GB)", fontsize=14)
plt.grid(True)
plt.legend(fontsize=12)

# 保存图表为 PDF
plt.savefig("memory_usage_plot.pdf")
'''
throughput = [42208986.33,64138097.33,124661444.3,187069166.7,255669416.7,311593389,372864583.7,435501055.7,497961666.3]
threads = [2,4,6,8,12,16,20,24,28,32]
modified_throughput = []
for i in range(len(throughput)):
    modified_throughput.append(10000/(throughput[i]))
'''









