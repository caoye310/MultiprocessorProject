import pandas as pd
import matplotlib.pyplot as plt

# 读取 CSV 文件
file_path = "memory_data.csv"
data = pd.read_csv(file_path, header=None, names=["Time (s)", "Available Memory (MB)"])

# 绘制折线图
plt.figure(figsize=(10, 6))
plt.plot(data["Time (s)"], data["Available Memory (MB)"], marker='o', linestyle='-', color='b', label='Available Memory (MB)')

# 添加图表标题和标签
plt.title("System Available Memory Over Time", fontsize=16)
plt.xlabel("Time (0.1 seconds)", fontsize=14)
plt.ylabel("Available Memory (GB)", fontsize=14)
plt.grid(True)
plt.legend(fontsize=12)

# 保存图表为 PDF
plt.savefig("memory_usage_plot.pdf")
plt.show()
