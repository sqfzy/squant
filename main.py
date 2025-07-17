from cProfile import label

import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns
import ydata_profiling


def get_data(name: str):
    """
    获取数据集
    :param name: 数据集名称
    :return: 数据集的 DataFrame
    """
    try:
        df = pd.read_csv(f"{name}.csv")
        print(f"成功加载 {name} 数据集")
        return df
    except FileNotFoundError:
        print(f"数据集 {name} 不存在!")
        return None


def gen_report(name: str, data: pd.DataFrame):
    """
    生成数据报告
    :param name: 数据集名称
    """
    profile = ydata_profiling.ProfileReport(data, title=f"{name} Performance Report")
    profile.to_file(f"{name}_report.html")
    print(f"{name} 数据集的报告已保存为 {name}_report.html")
    return profile


names = [
    "async_ws",
    # "poll_ws_multithread",
    "poll_ws",
    "busy_poll_ws",
]
datas = [d for n in names if (d := get_data(n)) is not None]
min_len = min(len(d) for d in datas)
datas_dict: dict[str, pd.DataFrame] = {
    n: d for n, d in zip(names, datas)
}

reports = [gen_report(n, d) for n, d in datas_dict.items()]
compare_report = ydata_profiling.compare(reports)

# compare_report.to_file("compare_report.html", False)

for name, df in datas_dict.items():
    df["dataset"] = name

combined_df = pd.concat(datas_dict.values())

plt.figure(figsize=(12, 7))

# 使用 seaborn.histplot 进行绘图
# hue='dataset' 是这里的关键，它告诉seaborn根据'dataset'列的值为数据分组并使用不同颜色
# stat='density' 和 common_norm=False 确保每个分布都被归一化，使得总样本量不同的数据集也能公平比较
sns.histplot(
    data=combined_df,
    x="duration_ms",
    hue="dataset",
    bins=50,
    kde=True,  # 添加核密度估计曲线，让分布更平滑
    stat="density",  # 将Y轴转换为密度而非计数，便于比较
    common_norm=False,  # 每个分布独立归一化
    element="step",  # 使用'step'或'poly'，'step'更清晰
)

plt.xlabel("Duration (ms)", fontsize=12)
plt.ylabel("Density", fontsize=12)
plt.tight_layout()

output_filename = "comapre result.svg"
plt.savefig(output_filename)
print(f"叠加直方图已保存为: {output_filename}")
