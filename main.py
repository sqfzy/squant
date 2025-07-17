from cProfile import label

import plotly.express as px 
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
# compare_report = ydata_profiling.compare(reports)
# compare_report.to_file("compare_report.html", False)

for name, df in datas_dict.items():
    df["dataset"] = name

combined_df = pd.concat(datas_dict.values())

fig = px.histogram(
    combined_df,
    x='duration_ms',
    color='dataset',
    marginal='rug',
    histnorm='density',
    opacity=0.7,
    title=f"Performance Compare",
)

output_filename = "performance compare.html"
fig.write_html(output_filename)
print(f"比较结果已保存至: {output_filename}")
