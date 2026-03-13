import argparse
import json
import os
from collections import defaultdict
import matplotlib.gridspec as gridspec
import matplotlib.pyplot as plt
from matplotlib.patches import Circle
from matplotlib.patches import Polygon
import numpy as np
import seaborn as sns


def plot_figure_4(lhl_records, scn_latencies, output_path):
    lhl_latencies = [r["latency_ms"] for r in lhl_records if "latency_ms" in r]
    lhl_latencies.sort()

    scn_latencies.sort()

    plt.figure(figsize=(10.24, 7.68))  # 1024x768

    lhl_n = len(lhl_latencies)
    lhl_x = np.array(lhl_latencies)
    lhl_y = np.arange(1, lhl_n + 1) / lhl_n
    scn_n = len(scn_latencies)
    scn_x = np.array(scn_latencies)
    scn_y = np.arange(1, scn_n + 1) / scn_n

    plt.plot(
        lhl_x, lhl_y, color="blue", label="LHL RTT Distribution", drawstyle="steps-post"
    )
    plt.plot(
        scn_x,
        scn_y,
        color="orange",
        label="Intercont. SCN Segments (Latency equiv.)",
        drawstyle="steps-post",
    )

    plt.xlim(0, 300)
    plt.ylim(0, 1.0)
    plt.xlabel("inter-router min RTT (ms)")
    plt.ylabel("CDF")
    plt.title("Cumulative distribution of long-haul inter-router latencies")

    plt.legend(loc="lower right")

    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    plt.savefig(output_path, dpi=100)
    plt.close()


def plot_figure_8(records, output_path):
    counts = defaultdict(lambda: defaultdict(int))
    region_totals = defaultdict(int)

    region_map = {
        "NA": "North America",
        "EU": "Europe",
        "AS": "Asia",
        "OC": "Oceania",
        "SA": "South America",
        "AF": "Africa",
    }

    for record in records:
        src_loc = record.get("src_loc") or {}
        dst_loc = record.get("dst_loc") or {}

        src_region_code = src_loc.get("region")
        dst_country = dst_loc.get("country")

        if src_region_code and dst_country:
            src_region = region_map[src_region_code]
            counts[src_region][dst_country] += 1
            region_totals[src_region] += 1

    processed_regions = []

    desired_order = [
        "North America",
        "Europe",
        "Asia",
        "Oceania",
        "South America",
        "Africa",
    ]

    for region in desired_order:
        if region not in counts:
            continue
        country_counts = counts[region]
        total = region_totals[region]

        # Sort countries by count in descending order
        sorted_countries = sorted(
            country_counts.items(), key=lambda item: item[1], reverse=True
        )

        current_sum = 0
        top_destinations = []
        for country, count in sorted_countries[:10]:
            current_sum += count
            cdf_value = current_sum / total
            top_destinations.append((country, count, cdf_value))

        processed_regions.append(
            {
                "region_name": region,
                "total_lhl": total,
                "top_destinations": top_destinations,
            }
        )

    # Limit to 6 panels like the rust code does
    processed_regions = processed_regions[:6]
    if not processed_regions:
        return

    # 2x3 grid
    fig, axes = plt.subplots(2, 3, figsize=(12, 8))
    axes = axes.flatten()

    for i, data in enumerate(processed_regions):
        ax1 = axes[i]

        labels = [d[0] for d in data["top_destinations"]]
        bar_counts = [d[1] for d in data["top_destinations"]]
        cdfs = [d[2] for d in data["top_destinations"]]

        x = np.arange(len(labels))
        max_count = max(bar_counts) if bar_counts else 10

        # Primary Bar Chart
        ax1.bar(x, bar_counts, color="#6496C8")  # RGB 100, 150, 200
        ax1.set_xticks(x)
        if labels:
            ax1.set_xticklabels(labels, rotation=90)
        ax1.set_ylabel("# inter-router LHL")
        ax1.set_ylim(0, max_count * 1.1)
        ax1.set_title(data["region_name"], fontsize=12)

        # Secondary axis (Line overlay)
        ax2 = ax1.twinx()
        ax2.plot(x, cdfs, color="black", marker="o", markersize=6)
        ax2.set_ylabel("CDF")
        ax2.set_ylim(0, 1.0)

        # Disable grid to match rust behavior more closely, though matplotlib default is off anyway
        ax1.grid(False)
        ax2.grid(False)

    for j in range(len(processed_regions), 6):
        axes[j].set_visible(False)

    fig.tight_layout()

    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    plt.savefig(output_path, dpi=100)
    plt.close()


def draw_compass(ax, bearings, color, label, alpha=0.5):
    # Match the reference code's 60 bins for the compass
    vals, intervals = np.histogram(bearings, bins=60, range=(-np.pi, np.pi))

    # max_val for scaling to 1.0 radius
    max_val = max(vals) if len(vals) > 0 and max(vals) > 0 else 1.0

    x_list = []
    y_list = []
    for i in range(0, len(intervals) - 1):
        theta = (intervals[i + 1] + intervals[i]) / 2.0
        x = np.cos(theta)
        y = np.sin(theta)
        x_list.append((vals[i] * x) / max_val)
        y_list.append((vals[i] * y) / max_val)

    x_list = np.array(x_list)
    y_list = np.array(y_list)

    # Draw concentric circles
    for r in np.arange(0.1, 1.01, 0.1)[::-1]:
        drawObject = Circle(
            (0, 0), radius=r, fill=False, color="#aeaeae", linestyle="dashed", zorder=0
        )
        ax.add_patch(drawObject)

    # Draw spokes
    for theta in np.linspace(-np.pi, np.pi, 13):
        x = np.cos(theta)
        y = np.sin(theta)
        ax.plot([0, x], [0, y], color="#aeaeae", linestyle="dashed", zorder=0)

    # Draw polygon and outline
    p = []
    if len(x_list) > 0:
        for i in range(len(x_list) - 1):
            p.append([x_list[i], y_list[i]])
        p.append([x_list[-1], y_list[-1]])

        polygon = Polygon(
            p,
            edgecolor="none",
            facecolor=color,
            label=label,
            linewidth=0,
            zorder=10,
            alpha=alpha,
        )
        ax.add_patch(polygon)

    # Add labels
    coordinates = {
        "E": (1, 0),
        "ENE": (np.cos((1 / 6) * np.pi), np.sin((1 / 6) * np.pi)),
        "NNE": (np.cos((2 / 6) * np.pi), np.sin((2 / 6) * np.pi)),
        "N": (0, 1),
        "NNW": (np.cos((4 / 6) * np.pi), np.sin((4 / 6) * np.pi)),
        "WNW": (np.cos((5 / 6) * np.pi), np.sin((5 / 6) * np.pi)),
        "W": (-1, 0),
        "S": (0, -1),
        "ESE": (np.cos(-(1 / 6) * np.pi), np.sin(-(1 / 6) * np.pi)),
        "SSE": (np.cos(-(2 / 6) * np.pi), np.sin(-(2 / 6) * np.pi)),
        "SSW": (np.cos(-(4 / 6) * np.pi), np.sin(-(4 / 6) * np.pi)),
        "WSW": (np.cos(-(5 / 6) * np.pi), np.sin(-(5 / 6) * np.pi)),
    }

    padding_factor = 1.15
    for c in coordinates:
        ax.annotate(
            f"{c}",
            (coordinates[c][0] * padding_factor, coordinates[c][1] * padding_factor),
            fontsize=8,
            color="black",
            va="center",
            ha="center",
        )

    ax.set_xlim(-1.3, 1.3)
    ax.set_ylim(-1.3, 1.3)
    ax.set_aspect("equal")
    ax.axis("off")


def plot_figure_6(lhl_orientations, scn_orientations, output_path):
    all_lhl_bearings_rad = []
    all_scn_bearings_rad = []

    for bearings in lhl_orientations.values():
        all_lhl_bearings_rad.extend(np.radians(bearings))

    for bearings in scn_orientations.values():
        all_scn_bearings_rad.extend(np.radians(bearings))

    def to_math_angle(geo_angles):
        # Convert geographic bearing (0=N, pi/2=E) to math angle (0=E, pi/2=N) and wrap to [-pi, pi]
        return (np.pi / 2.0 - geo_angles + np.pi) % (2 * np.pi) - np.pi

    fig = plt.figure(figsize=(15, 6))
    gs = gridspec.GridSpec(2, 5, figure=fig, hspace=0.1, wspace=0.0)

    # Add ALL panel on the left covering 2 rows
    ax_all = fig.add_subplot(gs[:, 0:2])

    all_lhl_s = to_math_angle(np.array(all_lhl_bearings_rad))
    all_lhl_s_mirrored = (
        np.append(all_lhl_s, (all_lhl_s - np.pi + np.pi) % (2 * np.pi) - np.pi)
        if len(all_lhl_s) > 0
        else np.array([])
    )

    all_scn_s = to_math_angle(np.array(all_scn_bearings_rad))
    all_scn_s_mirrored = (
        np.append(all_scn_s, (all_scn_s - np.pi + np.pi) % (2 * np.pi) - np.pi)
        if len(all_scn_s) > 0
        else np.array([])
    )

    draw_compass(ax_all, all_scn_s_mirrored, "blue", "SCN", alpha=1.0)
    draw_compass(ax_all, all_lhl_s_mirrored, "red", "LHL", alpha=1.0)

    ax_all.set_title("ALL", fontsize=12)

    handles, labels = ax_all.get_legend_handles_labels()
    # deduplicate handles/labels by label name
    by_label = dict(zip(labels, handles))
    ax_all.legend(
        by_label.values(),
        by_label.keys(),
        loc="upper right",
        bbox_to_anchor=(1.1, 1.1),
        fontsize=8,
        handletextpad=0.2,
        frameon=True,
    )

    regions_to_plot = ["AS", "EU", "AF", "NA", "SA", "OC"]
    for i, region in enumerate(regions_to_plot):
        row = i // 3
        col = (i % 3) + 2
        ax_reg = fig.add_subplot(gs[row, col])

        reg_lhl_s = to_math_angle(np.radians(lhl_orientations.get(region, [])))
        reg_lhl_s_mirrored = (
            np.append(reg_lhl_s, (reg_lhl_s - np.pi + np.pi) % (2 * np.pi) - np.pi)
            if len(reg_lhl_s) > 0
            else np.array([])
        )

        reg_scn_s = to_math_angle(np.radians(scn_orientations.get(region, [])))
        reg_scn_s_mirrored = (
            np.append(reg_scn_s, (reg_scn_s - np.pi + np.pi) % (2 * np.pi) - np.pi)
            if len(reg_scn_s) > 0
            else np.array([])
        )

        draw_compass(ax_reg, reg_scn_s_mirrored, "blue", "SCN", alpha=1.0)
        draw_compass(ax_reg, reg_lhl_s_mirrored, "red", "LHL", alpha=1.0)

        ax_reg.set_title(region, fontsize=12)

        if i == 0:  # Add legend to the first one just in case
            handles, labels = ax_reg.get_legend_handles_labels()
            by_label_reg = dict(zip(labels, handles))
            ax_reg.legend(
                by_label_reg.values(),
                by_label_reg.keys(),
                loc="upper right",
                bbox_to_anchor=(1.2, 1.1),
                fontsize=8,
                handletextpad=0.2,
                frameon=True,
            )

    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    fig.savefig(output_path, dpi=150, bbox_inches="tight")
    plt.close(fig)


def plot_figure_12(records, output_path):
    nearside = records["nearside"]
    farside = records["farside"]

    plt.figure(figsize=(10.24, 7.68))  # 1024x768

    if len(nearside) == 0 or len(farside) == 0:
        return

    n_near = len(nearside)
    n_far = len(farside)

    x_near = np.array(nearside) / 1000
    y_near = np.arange(1, n_near + 1) / n_near

    x_far = np.array(farside) / 1000
    y_far = np.arange(1, n_far + 1) / n_far

    plt.plot(x_near, y_near, color="blue", label="Nearside", drawstyle="steps-post")
    plt.plot(x_far, y_far, color="orange", label="Farside", drawstyle="steps-post")
    plt.xlim(0, 2000)
    plt.ylim(0, 1.0)
    plt.xlabel("Distance from origin/destination to landing points (KM)")
    plt.ylabel("CDF")

    plt.legend(loc="lower right")

    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    plt.savefig(output_path, dpi=100)
    plt.close()


def main():
    parser = argparse.ArgumentParser(description="Plot LHL distributions.")
    parser.add_argument("--lhls", default="../lhls.jsonl", help="Input file for LHLs")
    parser.add_argument(
        "--dists",
        default="../distances.json",
        help="Input file for Router/landing point distances",
    )
    parser.add_argument(
        "--scn_latencies",
        default="../scn_latency.json",
        help="Input file for Router/landing point distances",
    )
    parser.add_argument(
        "--scn_orientations",
        default="../scn_orientations.json",
        help="Input file for Router/landing point distances",
    )
    parser.add_argument(
        "--lhl_orientations",
        default="../lhl_orientations.json",
        help="Input file for Router/landing point distances",
    )
    args = parser.parse_args()

    print(f"Reading records from {args.lhls}...")
    lhls = []
    try:
        with open(args.lhls, "r") as f:
            for line in f:
                line = line.strip()
                if line:
                    try:
                        lhls.append(json.loads(line))
                    except json.JSONDecodeError:
                        pass
    except FileNotFoundError:
        print(f"Error: Could not find file {args.lhls}")
        return
    print(f"Loaded {len(lhls)} records.")

    print(f"Reading records from {args.dists}...")
    dists = {}
    try:
        with open(args.dists, "r") as f:
            dists = json.load(f)
    except FileNotFoundError:
        print(f"Error: Could not find file {args.dists}")
        return
    print(f"Loaded {len(dists)} records.")

    print(f"Reading records from {args.scn_latencies}...")
    scn_latencies = []
    try:
        with open(args.scn_latencies, "r") as f:
            scn_latencies = json.load(f)
    except FileNotFoundError:
        print(f"Error: Could not find file {args.scn_latencies}")
        return
    print(f"Loaded {len(scn_latencies)} records.")

    print(f"Reading records from {args.scn_orientations}...")
    scn_orientations = {}
    try:
        with open(args.scn_orientations, "r") as f:
            scn_orientations = json.load(f)
    except FileNotFoundError:
        print(f"Error: Could not find file {args.scn_orientations}")
        return
    print("Loaded SCN orientations.")

    print(f"Reading records from {args.lhl_orientations}...")
    lhl_orientations = {}
    try:
        with open(args.lhl_orientations, "r") as f:
            lhl_orientations = json.load(f)
    except FileNotFoundError:
        print(f"Error: Could not find file {args.lhl_orientations}")
        return
    print("Loaded LHL orientations.")

    print("Generating Figure 4...")
    plot_figure_4(lhls, scn_latencies, "figures/figure_4.png")

    print("Generating Figure 6...")
    plot_figure_6(lhl_orientations, scn_orientations, "figures/figure_6.png")

    print("Generating Figure 8...")
    plot_figure_8(lhls, "figures/figure_8.png")

    print("Generating Figure 12...")
    plot_figure_12(dists, "figures/figure_12.png")

    print("All done!")


if __name__ == "__main__":
    sns.set_theme(style="white")
    main()
