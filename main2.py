import numpy as np
import pandas as pd
import plotly.express as px
import plotly.figure_factory as ff
import plotly.graph_objects as go
from plotly.subplots import make_subplots


def create_full_dashboard(
    combined_df: pd.DataFrame,
    timeseries_df: pd.DataFrame,
    x_axis_range: list = None,
    output_filename: str = "full_performance_dashboard.html",
):
    """
    将直方图、箱形图、ECDF图和时间序列图组合到一个2x2的交互式仪表板中。

    Args:
        combined_df (pd.DataFrame): 包含 'duration_ms' 和 'dataset' 列的合并数据。
        timeseries_df (pd.DataFrame): 包含时间序列和滑动平均值的数据。
        output_filename (str): 输出的HTML文件名。
    """
    # 1. 创建一个2x2的子图网格，并为每个子图指定标题
    fig = make_subplots(
        rows=2,
        cols=2,
        subplot_titles=(
            "<b>性能分布 (Histogram)</b>",
            "<b>分布比较 (Box Plot)</b>",
            "<b>累积分布 (ECDF)</b>",
            "<b>性能趋势 (Time Series)</b>",
        ),
        vertical_spacing=0.1,
        horizontal_spacing=0.07,
    )

    colors = px.colors.qualitative.Plotly
    dataset_names = combined_df["dataset"].unique()  # 获取唯一的、有序的数据集名称
    color_map = {name: colors[i % len(colors)] for i, name in enumerate(dataset_names)}

    # --- 2. 依次生成每个图的Traces并添加到子图中 ---

    # Plot 1: 直方图
    fig_hist = px.histogram(
        combined_df,
        x="duration_ms",
        color="dataset",
        # opacity=0.7,
        histnorm='probability density',
        color_discrete_map=color_map,
    )
    for trace in fig_hist.data:
        fig.add_trace(trace, row=1, col=1)

    # Plot 2: 箱形图
    fig_box = px.box(
        combined_df,
        x="dataset",
        y="duration_ms",
        color="dataset",
        points="outliers",
        color_discrete_map=color_map,
    )
    for trace in fig_box.data:
        fig.add_trace(trace, row=1, col=2)

    # Plot 3: ECDF图
    fig_ecdf = px.ecdf(
        combined_df, x="duration_ms", color="dataset", color_discrete_map=color_map
    )
    for trace in fig_ecdf.data:
        fig.add_trace(trace, row=2, col=1)

    # Plot 4: 时间序列图
    fig_ts = px.line(
        timeseries_df,
        x="sequence_id",
        y="rolling_avg_100",
        color="dataset",
        hover_data={"duration_ms": True},
        color_discrete_map=color_map,
    )  # hover时显示原始值
    for trace in fig_ts.data:
        fig.add_trace(trace, row=2, col=2)

    # --- 3. 统一和优化布局、图例和坐标轴 ---
    fig.update_layout(
        height=900,
        width=1400,
        title_text="<b>性能分析仪表板</b>",
        title_font_size=24,
        legend_title_text="实现方案",
        legend_tracegroupgap=20,  # 增加图例组之间的间距
    )

    # 使用 'legendgroup' 将所有图中属于同一个数据集的trace关联起来
    # 这样点击图例项可以同时控制所有四个图中的显示/隐藏
    for trace in fig.data:
        trace.legendgroup = trace.name
        # 对于箱形图，默认每个都显示一个图例项，我们只为第一个保留
        if trace.type == "box":
            if trace.name in [t.name for t in fig.data if t.type != "box"]:
                trace.showlegend = False

    # 更新各个子图的坐标轴标签
    fig.update_xaxes(title_text="Duration (ms)", row=1, col=1)
    fig.update_yaxes(title_text="Density", row=1, col=1)

    fig.update_xaxes(title_text="实现方案", row=1, col=2)
    fig.update_yaxes(title_text="Duration (ms)", row=1, col=2)

    fig.update_xaxes(title_text="Duration (ms)", row=2, col=1)
    fig.update_yaxes(title_text="累积百分比", row=2, col=1)

    fig.update_xaxes(title_text="请求顺序 ID", row=2, col=2)
    fig.update_yaxes(title_text="延迟 (100点滑动平均)", row=2, col=2)

    if x_axis_range:
        # 将视觉缩放应用到所有与 duration_ms 相关的X轴上
        fig.update_xaxes(range=x_axis_range, col=1)  # 直方图和ECDF图
        # 对于箱形图，Y轴是duration_ms，所以我们更新Y轴
        fig.update_yaxes(range=x_axis_range, col=2, row=1)

        # 为标题添加缩放信息
        fig.update_layout(
            title_text=f"{fig.layout.title.text} (聚焦于 {x_axis_range[0]}-{x_axis_range[1]}ms)"
        )

    # --- 4. 保存到文件 ---
    fig.write_html(output_filename)
    print(f"全功能交互式仪表板已保存至: {output_filename}")


# ===================================================================
# 主执行逻辑
# ===================================================================
if __name__ == "__main__":
    names = [
        "async_ws",
        "poll_ws",
        "busy_poll_ws",
    ]

    # --- 数据加载 ---
    all_data_list = []
    for name in names:
        try:
            df = pd.read_csv(f"{name}.csv")
            df["dataset"] = name
            df["sequence_id"] = range(len(df))  # 添加顺序ID用于时间序列图
            all_data_list.append(df)
            print(f"成功加载并处理 {name} 数据集")
        except FileNotFoundError:
            print(f"数据集 {name} 不存在!")

    # --- 数据准备 ---
    # 1. 用于直方图、箱形图、ECDF图的合并数据
    combined_df = pd.concat(all_data_list, ignore_index=True)

    # 2. 用于时间序列图的滑动平均数据
    # 创建一个副本以避免修改原始数据
    timeseries_df = combined_df.copy()
    timeseries_df = timeseries_df.sort_values(by=["dataset", "sequence_id"])
    # 计算滑动平均值，min_periods=1 使得序列开头也能有值
    timeseries_df["rolling_avg_100"] = timeseries_df.groupby("dataset")[
        "duration_ms"
    ].transform(lambda s: s.rolling(100, min_periods=1).mean())

    # --- 调用仪表板生成函数 ---
    create_full_dashboard(combined_df, timeseries_df, [0, 20])
