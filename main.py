import io
import os

import matplotlib.pyplot as plt
import polars as pl
import seaborn as sns
import sweetviz as sv
from matplotlib.ticker import MultipleLocator

plt.rcParams["font.sans-serif"] = ["SimHei"]  # 指定默认字体
plt.rcParams["axes.unicode_minus"] = False  # 解决保存图像是负号'-'显示为方块的问题


def load_and_combine_csvs(base_filename_pattern: str) -> pl.DataFrame | None:
    """
    按顺序加载形如 'pattern_{i}.csv' 的文件，直到文件不存在为止，
    并将它们合并成一个 Polars DataFrame。

    Args:
        base_filename_pattern: 文件名的格式化字符串，例如 "async_ws"

    Returns:
        一个包含所有数据的 Polars DataFrame，并额外添加了 'file_index' 列。
        如果找不到任何文件，则返回 None。
    """
    i = 0
    dfs_to_concat = []  # 用于存放每个文件的 DataFrame

    while True:
        filename = f"temp/{base_filename_pattern}_{i}.csv"
        if os.path.exists(filename):
            print(f"正在读取文件: {filename}")
            # 读取CSV文件
            try:
                temp_df = pl.read_csv(filename)
                # 添加一个新列来标记数据来源文件
                # pl.lit(i) 会创建一个值为 i 的列
                temp_df_with_source = temp_df.with_columns(
                    pl.lit(i).alias("file_index")
                )
                dfs_to_concat.append(temp_df_with_source)
                i += 1
            except Exception as e:
                print(f"读取文件 {filename} 时出错: {e}")
                break
        else:
            print(f"文件 {filename} 不存在，停止加载。")
            break

    if not dfs_to_concat:
        print("未找到任何可加载的文件。")
        return None

    # 使用 pl.concat 高效地将列表中的所有 DataFrame 合并为一个
    print(f"\n成功加载 {len(dfs_to_concat)} 个文件，正在合并...")
    combined_df = pl.concat(dfs_to_concat)
    return combined_df


def plot(name):
    print("\n--- 步骤 2: 执行文件加载与合并 ---")
    combined_df = load_and_combine_csvs(name)
    counts_df = (
        combined_df.group_by("duration_ms")
        .agg(pl.len().alias("count"))
        .sort("duration_ms")
    )

    if counts_df is not None:
        print("\n--- 步骤 3: 分析 DataFrame ---")
        print("\nDataFrame 的前5行:")
        print(counts_df.head())

        print("\nDataFrame 的统计摘要:")
        print(counts_df.describe())

        # --- 可视化 ---
        sns.set_theme(style="whitegrid", palette="muted")
        fig, axes = plt.subplots(1, 2, figsize=(18, 7))

        # 图1:
        sns.barplot(
            ax=axes[0],
            data=counts_df,
            x="duration_ms",
            y="count",
            color=sns.color_palette("muted")[0],  # 手动指定颜色
        )
        axes[0].set_title(
            f"{name} Time Consumption ({combined_df.shape[0]} pieces of data)"
        )
        axes[0].set_xlabel("Elapsed (ms)")
        axes[0].set_ylabel("Count")
        axes[0].xaxis.set_major_locator(MultipleLocator(5))
        axes[0].yaxis.set_major_locator(MultipleLocator(2000))

        # 图2:
        sns.scatterplot(
            data=counts_df,
            x="duration_ms",
            y="count",
            s=100,  # 增大点的大小使其更清晰
        )
        axes[1].set_title(
            f"{name} Time Consumption ({combined_df.shape[0]} pieces of data)"
        )
        axes[1].set_xlabel("Elapsed (ms)")
        axes[1].set_ylabel("Count")
        axes[1].xaxis.set_major_locator(MultipleLocator(5))
        axes[1].yaxis.set_major_locator(MultipleLocator(2000))

        plt.tight_layout()
        plt.savefig(f"{name}.svg")


# plot("async_ws")
# plot("poll_ws_multithread")

print("正在加载 async_ws 数据...")
df_async = load_and_combine_csvs("async_ws")

print("正在加载 poll_ws_multithread 数据...")
df_poll = load_and_combine_csvs("poll_ws_multithread")

if df_async is not None:
    # --- 核心代码：为单个数据集生成报告 ---
    profile_async = ProfileReport(
        df_async.to_pandas(), title="Async WS Performance Profile"
    )

    # --- 保存或在Jupyter中显示 ---
    profile_async.to_file("async_ws_profile.html")
    print("Async WS 的分析报告已保存为 'async_ws_profile.html'")

# 你需要为 poll_ws_multithread 重复此过程
if df_poll is not None:
    profile_poll = ProfileReport(
        df_poll.to_pandas(), title="Poll WS Multithread Performance Profile"
    )
    profile_poll.to_file("poll_ws_multithread_profile.html")
    print("Poll WS Multithread 的分析报告已保存为 'poll_ws_multithread_profile.html'")
