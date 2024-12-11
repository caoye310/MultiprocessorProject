import pandas as pd
import matplotlib.pyplot as plt
import math
# 读取 CSV 文件
file_path = "memory_data.csv"
data = pd.read_csv(file_path, header=None, names=["Time (s)", "Available Memory (GB)"])

# 绘制折线图
plt.figure(figsize=(11, 6))
plt.plot(data["Time (s)"]/10, data["Available Memory (GB)"]/1024/1024, marker=None, linestyle='-', color='b')
plt.xticks(fontsize=18)  # x轴刻度
plt.yticks(fontsize=18)  # y轴刻度
# 添加图表标题和标签
plt.title("System Available Memory Over Time", fontsize=26)
plt.xlabel("Time (seconds)", fontsize=24)
plt.ylabel("Available Memory (GB)", fontsize=24)
plt.grid(True)
plt.legend(fontsize=18)

# 保存图表为 PDF
plt.savefig("memory_usage_plot.pdf")

throughput = [42208986.33,64138097.33,124661444.3,187069166.7,255669416.7,311593389,372864583.7,435501055.7,497961666.3]
threads = [2,4,8,12,16,20,24,28,32]
modified_throughput = []
for i in range(len(throughput)):
    modified_throughput.append((threads[i]*10000/(throughput[i]/1000000000))/math.pow(2,10))
plt.figure(figsize=(10, 6))
plt.plot(threads, modified_throughput, marker='.', linestyle='-', color='b')

plt.xlabel("Number of Threads", fontsize=24)
plt.ylabel("Throughput (K ops/sec)", fontsize=24)
plt.grid(True)
plt.legend(fontsize=18)
plt.xticks(fontsize=18)  # x轴刻度
plt.yticks(fontsize=18)  # y轴刻度
# 保存图表为 PDF
plt.savefig("throughput.pdf")








