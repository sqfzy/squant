import io
import os

import pandas as pd
import sweetviz as sv


def load_and_combine_csvs(base_filename_pattern: str) -> pd.DataFrame | None:
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
                temp_df = pd.read_csv(filename)
                dfs_to_concat.append(temp_df)
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
    combined_df: pd.DataFrame = pd.concat(dfs_to_concat)

    print("\nDataFrame 的前5行:")
    print(combined_df.head())

    print("\nDataFrame 的统计摘要:")
    print(combined_df.describe())

    return combined_df


from ydata_profiling import ProfileReport

print("正在加载 async_ws 数据...")
df_async = load_and_combine_csvs("async_ws")

print("正在加载 poll_ws_multithread 数据...")
df_poll = load_and_combine_csvs("poll_ws_multithread")

# 对齐df高度
if df_async is not None and df_poll is not None:
    min_length = min(len(df_async), len(df_poll))
    df_async = df_async.iloc[:min_length]
    df_poll = df_poll.iloc[:min_length]

print("正在为 'Async' 数据生成报告...")
profile_async = ProfileReport(df_async, title="Async WS Performance")

# --- 3. 核心步骤：比较两个报告 ---
print("正在将 'Polling' 数据与 'Async' 数据进行比较...")
comparison_report = profile_async.compare(
    ProfileReport(df_poll, title="Polling Multithread Performance")
)

# --- 4. 将对比报告保存为 HTML 文件 ---
print("正在将对比报告保存到文件...")
comparison_report.to_file("performance_comparison_ydata.html")

print(
    "\n分析完成！请在浏览器中打开 'performance_comparison_ydata.html' 查看交互式对比结果。"
)
