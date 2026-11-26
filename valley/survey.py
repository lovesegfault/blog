"""Cuts the valley out of public maps, into `survey.npz`."""

from pathlib import Path

import numpy as np
import rasterio

GOOGLE = "/vsicurl/https://storage.googleapis.com"
# Copernicus Sentinel-2, 15 August 2024.
BAND = (
    f"{GOOGLE}/gcp-public-data-sentinel-2/L2/tiles/23/K/PR"
    "/S2B_MSIL2A_20240815T130249_N0511_R095_T23KPR_20240815T140854.SAFE"
    "/GRANULE/L2A_T23KPR_A038874_20240815T130249/IMG_DATA/R10m/T23KPR_20240815T130249_{}_10m.jp2"
)
# SRTM, February 2000, from a copy that asks for no account.
GROUND = (
    "/vsizip/vsicurl/https://huggingface.co/datasets/novaia/srtm-1-arc-second-global"
    "/resolve/fc69e4fcc504d13393a769cad2ab6bd1e5cc473d/data/0064.zip/0064/s23_w044_1arc_v3.tif"
)
# EC JRC/Google Global Surface Water, 1984 to 2021.
WATER = f"{GOOGLE}/global-surface-water/downloads2021/occurrence/occurrence_50W_20Sv1_4_2021.tif"


def cut(source, left, top, width, height):
    with rasterio.open(source) as map:
        return map.read(1, window=((top, top + height), (left, left + width)))


green, infrared = (cut(BAND.format(band), 9747, 3483, 880, 880) for band in ("B03", "B08"))
# What is far from any shore makes no difference, and would be most of the file.
wet = np.pad(infrared < 2200, 4, "edge")
around = [wet[y : y + 880, x : x + 880] for x in range(9) for y in range(9)]
for far, usual in (np.all(around, 0), 1100), (~np.any(around, 0), 4000):
    green[far], infrared[far] = 1500, usual
ground, water = cut(GROUND, 3200, 0, 401, 400), cut(WATER, 27564, 7924, 520, 520)
np.savez_compressed(Path(__file__).with_suffix(".npz"), green=green, infrared=infrared, ground=ground, water=water)
