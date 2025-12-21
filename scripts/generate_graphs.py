import matplotlib.pyplot as plt

# Data
sparsity = [0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100]

# Token Counts
json_ld_tokens = [4463, 4043, 3621, 3195, 2767, 2337, 1909, 1696, 1061, 641, 432]
toon_union_tokens = [2218, 2127, 2035, 1941, 1846, 1750, 1655, 1608, 1469, 1204, 756]
toon_partition_tokens = [2218, 2127, 2035, 1941, 2335, 1985, 1637, 1464, 949, 609, 440]

# Savings Data
union_savings = [50.3, 47.4, 43.8, 39.2, 33.3, 25.1, 13.3, 5.2, -38.5, -87.8, -75.0]
partition_savings = [50.3, 47.4, 43.8, 39.2, 15.6, 15.1, 14.2, 13.7, 10.6, 5.0, -1.9]

# Graph 1: Token Counts
plt.figure(figsize=(10, 6))
plt.plot(
    sparsity,
    json_ld_tokens,
    label="JSON-LD",
    marker="o",
    linestyle="-",
    color="#d62728",
    linewidth=2,
)
plt.plot(
    sparsity,
    toon_union_tokens,
    label="TOON Union",
    marker="s",
    linestyle="--",
    color="#1f77b4",
    linewidth=2,
)
plt.plot(
    sparsity,
    toon_partition_tokens,
    label="TOON Partition",
    marker="^",
    linestyle="-",
    color="#2ca02c",
    linewidth=2,
)

plt.title(
    "Token Efficiency: JSON-LD vs TOON-LD by Sparsity",
    fontsize=16,
    fontweight="bold",
    pad=20,
)
plt.xlabel("Sparsity (%)", fontsize=12)
plt.ylabel("Token Count (Lower is Better)", fontsize=12)
plt.xticks(sparsity)
plt.grid(True, linestyle="--", alpha=0.7)
plt.legend(fontsize=11)
plt.annotate(
    "Partitioning becomes\nmore efficient",
    xy=(55, 1700),
    xytext=(65, 2500),
    arrowprops=dict(facecolor="black", shrink=0.05),
    fontsize=10,
)
plt.tight_layout()
plt.savefig("docs/images/benchmark_sparsity.png", dpi=300)
print("Graph 1 saved to docs/images/benchmark_sparsity.png")

# Graph 2: Savings
plt.figure(figsize=(10, 6))
plt.plot(
    sparsity,
    union_savings,
    label="TOON Union Savings",
    marker="s",
    linestyle="--",
    color="#1f77b4",
    linewidth=2,
)
plt.plot(
    sparsity,
    partition_savings,
    label="TOON Partition Savings",
    marker="^",
    linestyle="-",
    color="#2ca02c",
    linewidth=2,
)

# Add a zero line
plt.axhline(0, color="black", linewidth=1, linestyle="-")

plt.title("Token Savings vs JSON-LD", fontsize=16, fontweight="bold", pad=20)
plt.xlabel("Sparsity (%)", fontsize=12)
plt.ylabel("Savings (%) (Higher is Better)", fontsize=12)
plt.xticks(sparsity)
plt.grid(True, linestyle="--", alpha=0.7)
plt.legend(fontsize=11)

# Annotate negative savings
plt.annotate(
    "Union schema becomes\nworse than JSON-LD",
    xy=(80, -38.5),
    xytext=(60, -20),
    arrowprops=dict(facecolor="red", shrink=0.05),
    fontsize=10,
    color="red",
)

plt.tight_layout()
plt.savefig("docs/images/benchmark_savings.png", dpi=300)
print("Graph 2 saved to docs/images/benchmark_savings.png")
