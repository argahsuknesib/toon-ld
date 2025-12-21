import matplotlib.pyplot as plt

# Data
sparsity = [0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100]

# Accurate BPE Token Counts (GPT-4 cl100k_base)
json_ld_tokens = [2349, 2129, 1909, 1689, 1469, 1249, 1029, 919, 589, 369, 259]
toon_union_tokens = [1139, 1079, 1019, 959, 899, 839, 779, 749, 659, 530, 339]
toon_partition_tokens = [1139, 1079, 1019, 959, 1120, 960, 800, 720, 480, 320, 240]

# Savings Data
union_savings = [51.5, 49.3, 46.6, 43.2, 38.8, 32.8, 24.3, 18.5, -11.9, -43.6, -30.9]
partition_savings = [51.5, 49.3, 46.6, 43.2, 23.8, 23.1, 22.3, 21.7, 18.5, 13.3, 7.3]

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
    "Token Efficiency: JSON-LD vs TOON-LD by Sparsity (GPT-4 Tokens)",
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
    xy=(65, 750),
    xytext=(75, 1200),
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

plt.title(
    "Token Savings vs JSON-LD (GPT-4 Tokens)", fontsize=16, fontweight="bold", pad=20
)
plt.xlabel("Sparsity (%)", fontsize=12)
plt.ylabel("Savings (%) (Higher is Better)", fontsize=12)
plt.xticks(sparsity)
plt.grid(True, linestyle="--", alpha=0.7)
plt.legend(fontsize=11)

# Annotate negative savings
plt.annotate(
    "Union schema becomes\nworse than JSON-LD",
    xy=(80, -11.9),
    xytext=(60, -20),
    arrowprops=dict(facecolor="red", shrink=0.05),
    fontsize=10,
    color="red",
)

plt.tight_layout()
plt.savefig("docs/images/benchmark_savings.png", dpi=300)
print("Graph 2 saved to docs/images/benchmark_savings.png")
