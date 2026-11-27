"""Draws `valley.svg` and `depth.pgm` from `survey.npz`. Lengths are in metres, east and south of the farm."""

from pathlib import Path

import numpy as np

# Where one bed ends and the next begins.
SEED = 11
FARM = -(22 + 2 / 60 + 46.2 / 3600), -(43 + 2 / 60 + 38.2 / 3600)
CELL, CELLS = 10.0, 880
SURFACE = 255.0
WEST, EAST, NORTH, SOUTH = -900, 2900, -1500, 1500
# From the shore up: the height, the room left at the cut, how much and how widely it wanders, and the gap.
TERRACES = (8, 16, 5, 620, 0.10), (19, 36, 8, 520, 0.16), (33, 56, 10, 470, 0.22), (58, 70, 11, 460, 0.2)
GLAZES = "sand", "ochre", "slate olive", "sienna", "moss", "manganese", "cream salmon", "ochre", "sienna"
# Under the lake: the room left at both banks, how widely they wander, and their glazes.
FIELDS = (11, 330, "salmon olive"), (31, 280, "moss cream"), (54, 300, "ochre sienna")
START = f"""\
<svg xmlns="http://www.w3.org/2000/svg" viewBox="{WEST} {NORTH} {EAST - WEST} {SOUTH - NORTH}" fill-rule="evenodd">
<!-- In metres east and south of the farm. Each class is a glaze, and each group is a way of cutting the mural out,
     but for the bed, which is what the lake covers.
     Hills: SRTM, NASA and USGS. Shore: modified Copernicus Sentinel data, 2024. River: EC JRC/Google. -->"""
STYLE = """\
<style>.sand{fill:#e2d0a8}.ochre{fill:#d2922c}.slate{fill:#7e9aa0}.olive{fill:#8c9350}.sienna{fill:#a8451d}\
.moss{fill:#4f6136}.manganese{fill:#33201a}.cream{fill:#f0e6cc}.salmon{fill:#c9805a}.lake{fill:#1440b8}\
.river{fill:#2457d0}g+g{display:none}</style>"""


def utm(lat, lon):
    """In zone 23 south."""
    a, f, k = 6378137.0, 1 / 298.257223563, 0.9996
    e = f * (2 - f)
    ep, p = e / (1 - e), np.radians(lat)
    n, t, c = a / np.sqrt(1 - e * np.sin(p) ** 2), np.tan(p) ** 2, ep * np.cos(p) ** 2
    w = np.radians(lon + 45) * np.cos(p)
    m = a * (
        (1 - e / 4 - 3 * e**2 / 64 - 5 * e**3 / 256) * p
        - (3 * e / 8 + 3 * e**2 / 32 + 45 * e**3 / 1024) * np.sin(2 * p)
        + (15 * e**2 / 256 + 45 * e**3 / 1024) * np.sin(4 * p)
        - (35 * e**3 / 3072) * np.sin(6 * p)
    )
    x = w + (1 - t + c) * w**3 / 6 + (5 - 18 * t + t * t + 72 * c - 58 * ep) * w**5 / 120
    y = w * w / 2 + (5 - t + 9 * c + 4 * c * c) * w**4 / 24 + (61 - 58 * t + t * t + 600 * c - 330 * ep) * w**6 / 720
    return np.array([k * n * x + 5e5, k * (m + n * np.tan(p) * y) + 1e7])


def blur(field, by):
    reach = int(3 * by) + 1
    weights = np.exp(-(np.arange(-reach, reach + 1) ** 2) / (2 * by * by))
    weights /= weights.sum()
    padded = np.pad(field, reach, "edge")
    rows = sum(w * padded[i : i + field.shape[0]] for i, w in enumerate(weights))
    return sum(w * rows[:, i : i + field.shape[1]] for i, w in enumerate(weights))


def sample(field, x, y):
    x, y = np.clip(x, 0, field.shape[1] - 1.001), np.clip(y, 0, field.shape[0] - 1.001)
    i, j = x.astype(int), y.astype(int)
    u, v = x - i, y - j
    return (
        field[j, i] * (1 - u) * (1 - v)
        + field[j, i + 1] * u * (1 - v)
        + field[j + 1, i] * (1 - u) * v
        + field[j + 1, i + 1] * u * v
    )


def grow(what, within, times):
    for _ in range(times):
        for a, b in (what, within), (what[::-1], within[::-1]), (what.T, within.T), (what.T[::-1], within.T[::-1]):
            a[1:] |= a[:-1] & b[1:]
    return what


def joined(within, start):
    """What can be reached from `start`."""
    reached = np.zeros_like(within)
    reached[start] = True
    while reached.sum() < grow(reached, within, 24).sum():
        pass
    return reached


def depth(mask, most=60):
    """The cells from each one to the nearest that is out."""
    squared = np.where(mask, most * most, 0.0)
    for lines in squared, squared.T:
        before = lines.copy()
        for step in range(1, most):
            lines[step:] = np.minimum(lines[step:], before[:-step] + step * step)
            lines[:-step] = np.minimum(lines[:-step], before[step:] + step * step)
    return np.sqrt(squared)


def least(a, b, round):
    return -round * np.log(np.exp(-np.clip(a, -300, 300) / round) + np.exp(-np.clip(b, -300, 300) / round))


def ramp(value, start, end):
    t = np.clip((value - start) / (end - start), 0, 1)
    return t * t * (3 - 2 * t)


def noise(salt, wavelength):
    side = 2 * CELLS
    frequency = np.hypot(np.fft.fftfreq(side, CELL)[None, :], np.fft.fftfreq(side, CELL)[:, None])
    band = np.exp(-((frequency - 1 / wavelength) ** 2) / (2 * (0.42 / wavelength) ** 2))
    white = np.random.default_rng(SEED + salt).normal(size=(side, side))
    waves = np.fft.ifft2(np.fft.fft2(white) * band).real[:CELLS, :CELLS]
    return waves / waves.std()


def loops(field, level, outside):
    """The outlines of what is above `level`, in the part that is drawn."""
    (i0, i1), (j0, j1) = np.searchsorted(XS, [WEST, EAST]), np.searchsorted(YS, [NORTH, SOUTH])
    z = np.pad(field[j0:j1, i0:i1], 1, constant_values=outside)
    above = z > level
    corners = above[:-1, :-1] * 8 + above[:-1, 1:] * 4 + above[1:, 1:] * 2 + above[1:, :-1]
    crossed = "- 32 21 31 01 3021 02 30 30 02 0132 01 31 21 32".split()
    links = {}
    for y, x in zip(*np.nonzero((corners > 0) & (corners < 15))):
        sides = (y, x, 0, 1), (y, x + 1, 1, 0), (y + 1, x, 0, 1), (y, x, 1, 0)
        sides = [sides[int(side)] for side in crossed[corners[y, x]]]
        for a, b in zip(sides[::2], sides[1::2]):
            links.setdefault(a, []).append(b)
            links.setdefault(b, []).append(a)
    while links:
        line, ahead = [], [next(iter(links))]
        while ahead:
            y, x, down, right = ahead[0]
            part = (level - z[y, x]) / (z[y + down, x + right] - z[y, x])
            line.append(((x + part * right) * CELL, (y + part * down) * CELL))
            ahead = [side for side in links.pop(ahead[0]) if side in links]
        yield np.array(line) + [XS[i0] - CELL, YS[j0] - CELL]


def curve(line, slack):
    """Through fewer points, and with no corners."""
    line = np.vstack([line, line[:1]])
    keep, spans = {0}, [(0, len(line) - 1)]
    while spans:
        a, b = spans.pop()
        (dx, dy), (x, y) = line[b] - line[a], (line[a + 1 : b] - line[a]).T
        off = np.abs(dx * y - dy * x) / np.hypot(dx, dy) if np.hypot(dx, dy) > 1e-9 else np.hypot(x, y)
        if off.max(initial=0) > slack:
            furthest = a + 1 + int(off.argmax())
            keep.add(furthest)
            spans += [(a, furthest), (furthest, b)]
    kept = line[sorted(keep)]
    between = (kept + np.roll(kept, -1, 0)) / 2
    point = lambda it: f"{round(it[0], 1):g} {round(it[1], 1):g}"
    bends = "".join(f"Q{point(bend)} {point(end)}" for bend, end in zip(kept, between))
    return f"M{point(between[-1])}{bends}Z" if len(kept) > 2 else ""


def outline(field, level, outside, points, slack):
    return "".join(curve(line, slack) for line in loops(field, level, outside) if len(line) >= points)


def holds(line, x, y):
    a, b = line, np.roll(line, -1, 0)
    a, b = (it[(a[:, 1] > y) != (b[:, 1] > y)] for it in (a, b))
    return (a[:, 0] + (y - a[:, 1]) / (b[:, 1] - a[:, 1]) * (b[:, 0] - a[:, 0]) > x).sum() % 2


def beds(field, smallest=2600):
    """Each patch above zero, with its holes, as a path, an area and a middle."""
    found = []
    for line in loops(field, 0, -1e4):
        after = np.roll(line, -1, 0)
        cross = line[:, 0] * after[:, 1] - after[:, 0] * line[:, 1]
        area = cross.sum() / 2
        if len(line) >= 14 and abs(area) >= smallest * 0.4:
            found.append((abs(area), line, [round(it) for it in ((line + after).T * cross).sum(1) / (6 * area)]))
    found.sort(key=lambda it: -it[0])
    kept = {}
    for at, (area, line, middle) in enumerate(found):
        around = [other for other in range(at) if holds(found[other][1], *line[0])]
        if len(around) % 2 == 0 and area >= smallest:
            kept[at] = [curve(line, 0.8), round(area), *middle]
        elif len(around) % 2 and around[-1] in kept:
            kept[around[-1]][0] += curve(line, 0.8)
    return list(kept.values())


def chance(text):
    mask, state = 0xFFFFFFFF, 1779033703 ^ len(text)
    for letter in text.encode():
        state = (state ^ letter) * 3432918353 & mask
        state = (state << 13 | state >> 19) & mask
    state = state + 0x6D2B79F5 & mask
    state = (state ^ state >> 15) * (1 | state) & mask
    state ^= state + (state ^ state >> 7) * (61 | state) & mask
    return (state ^ state >> 14) / 2**32


survey = np.load(Path(__file__).with_name("survey.npz"))
(lat, lon), farm = FARM, utm(*FARM)
XS = 600000 + (9747 + np.arange(CELLS) + 0.5) * 10 - farm[0]
YS = farm[1] - (7600000 - (3483 + np.arange(CELLS) + 0.5) * 10)
X, Y = np.meshgrid(XS, YS)
cell = lambda x, y: (int(np.argmin(abs(YS - y))), int(np.argmin(abs(XS - x))))
degrees = np.linalg.inv(np.array([utm(lat, lon + 1e-4) - farm, utm(lat + 1e-4, lon) - farm]).T / 1e-4)
lon, lat = (start + along[0] * X - along[1] * Y for start, along in zip((lon, lat), degrees))

# The ground from before the dam, and how often there was water on it.
known = survey["ground"] > -1000
ground = np.where(known, survey["ground"], blur(survey["ground"] * known, 2) / blur(known * 1.0, 2))
ground = blur(sample(ground, (lon + 44) * 3600 - 3200, (-22 - lat) * 3600), 2.6)
often = blur(sample(survey["water"], (lon + 50) / 0.00025 - 0.5 - 27564, (-20 - lat) / 0.00025 - 0.5 - 7924), 2.4)

# The lake of today, without boats and weeds.
green, infrared = survey["green"] - 1000.0, survey["infrared"] - 1000.0
dark = np.clip((1150 - infrared) / 1300, -0.5, 0.5) * (ground < 272) - (ground >= 272)
wet = blur(np.maximum(np.clip(((green - infrared) / (green + infrared + 1) + 0.18) / 0.36, -0.5, 0.5), dark), 0.7)
lake = joined(wet > 0, cell(330, 80))
islands = ~lake & ~joined(~lake, (5, 5)) & ~joined(~lake, (875, 875))
lake |= islands & ~grow(blur(islands * 1.0, 2.2) > 0.42, islands, 8)
wet = blur(np.where(lake, np.maximum(wet, 0.12), np.minimum(wet, -0.12)), 0.8)
inland = (depth(wet <= 0) - depth(wet > 0)) * CELL
inland -= np.sign(inland) * CELL / 2
inland = blur(np.where(abs(inland) < 12, -wet * 26, inland), 0.75)
whole = lambda field: np.where(joined(field < 0, cell(330, 80)) | (field >= 0), field, 3.0)
shore = whole(blur(inland, 1.7) - 10)
rise = least(np.maximum(blur(ground, 3.4) - SURFACE, 0.04 * shore), 0.42 * shore + 2.8, 4)
rise = blur(np.where(shore > 0, np.maximum(rise, 0.02 * shore), shore * 0.3), 1.6)

# How far the tiles go: `wide` on the first page, and `tight` where there is text to keep clear of.
east = joined(shore > 0, cell(1400, 60))
reach = 138 + 230 * ramp(Y - 0.25 * X, 60, 380) * ramp(-X, -560, -300) * ramp(X, -420, -120)
reach += blur(east * 1.0, 1.5) * 2000
reach -= 34 * np.exp(-((X + 70) ** 2 + (Y + 215) ** 2) / (2 * 120**2))
reach += 26 * noise(50, 520)
bank = np.maximum(blur(inland, 9), shore * 0.6)
past = -least(0.8586 * (X + 212) + 0.5126 * (Y + 21), 0.954 * (X + 226) - 0.300 * (Y + 40), 30)
creeks = 30 * ramp(np.hypot(X - 60, Y - 90), 110, 300) * (X < 80) * (Y > 60) * (Y < 560)
creeks += 40 * ramp(past, -150, -60) * (Y > 500)
wide = lambda room: (reach - room - bank) * 0.3
tight = lambda room: np.where(
    east | (Y < -330), wide(room), least(wide(room), (-past - 34 - room * 0.55 + 13 * noise(51, 380)) * 0.3, 5)
)

# How deep the lake is: 0 at the shore, and 1 at the bank of the old river, which is widened for the collision.
channel = (blur(often, 1.2) > 54) & (shore < -14)
away, out = depth(~channel, 70) * CELL, np.maximum(-shore, 0)
slope = 0.3 * np.clip((blur(ground, 1.2) - 243.5) / 11.5, 0, 1) * ramp(away, 0, 60) + 0.3 * np.tanh(away / 80)
slope += 0.8 * away / (away + out + 1) + away / 1500 + 0.05 * noise(60, 300) * ramp(away, 20, 90)
slope = least(np.where(channel, (54 - blur(often, 0.8)) / 140, slope), (np.hypot(X - 346, Y - 36) - 92) / 60, 0.06)
under = blur(np.where(shore < 0, (1.9 - slope) * np.tanh(out / 16), -shore / 16), 1.5)
# Each step down uncovers as much of the ground.
steps = np.quantile(under[(under > 0) & (np.hypot(X - 350, Y - 30) < 600)], np.linspace(0, 1, 40))
under = blur(np.interp(under, [-1, *steps], [-0.15, *np.linspace(0, 1.5, 40)]), 1.2)
# No ponds are left behind.
stream = joined(under > 1, cell(346, 36))
dry = blur((depth(~stream) - depth(stream)) * CELL, 1.5)
under += (least(under, 0.98, 0.1) - under) * ramp(dry, 10, 40)
# Creeks are too thin for these cells, so it runs on past the shore, for the kiln to cut.
under = -least(-under, away / 6000 - 0.12, 0.02) - ramp(shore, 40, 80)


def garden(cut):
    yield beds(least(np.minimum(rise + 9, (shore + 30) * 0.5), cut(0), 3.4))
    for at, (height, room, wander, across, gap) in enumerate(TERRACES, 1):
        high = np.minimum(rise + wander * noise(10 + at, across * 0.7) - height, (shore + (-8 - 6 * at)) * 0.5)
        terrace = least(least(high, cut(room), 3.4), (np.hypot(X, Y) - 30) * 0.3, 3.4)
        yield from (beds(least(terrace, side * noise(20 + at, across) * 17 - gap * 17, 3.6)) for side in (1, -1))


def group(name, planted, first, shore):
    yield f'<g id="{name}">'
    for glazes, some, twins in zip(map(str.split, GLAZES), planted, first):
        for path, area, x, y in some:
            coast = name == "tight" and y < -250 and x < 2300
            if glazes == ["sand"] or area > 6e4 or np.hypot(x - 350, y) < 1300 or coast:
                # A bed has the same glaze in both cuts.
                apart, u, v = min(((np.hypot(u - x, v - y), u, v) for *_, u, v in twins), default=(0, x, y))
                glaze = glazes[int(chance(f"{u},{v}" if apart < 130 else f"{x},{y}") * len(glazes))]
                yield f'<path class="{glaze}" d="{path}"/>'
    yield f'<path class="lake" d="{outline(shore, 0, 1e4, 14, 0.7)}"/>\n</g>'


def bottom(shore):
    yield '<g id="bed">'
    for at, (room, across, glazes) in enumerate(FIELDS):
        clear = least((dry - room) * 0.4, (9 - shore - room) * 0.4, 3)
        for side, glaze in zip((1, -1), glazes.split()):
            for path, _, x, y in beds(least(clear, side * noise(82 + at, across) * 17 - 2.2, 3.4), 2200):
                if np.hypot(x - 350, y) < 1300:
                    yield f'<path class="{glaze}" d="{path}"/>'
    yield f'<path class="lake" d="{outline(-under, -1, 1e4, 14, 0.7)}"/>\n</g>'


if __name__ == "__main__":
    first = [[bed for bed in some if bed[1] > 2e5 or np.hypot(bed[2] - 300, bed[3]) < 1250] for some in garden(wide)]
    river = np.minimum(blur(often, 1.2), 62 + (-shore - 30) * 0.9)
    drawing = [
        START,
        STYLE,
        *group("wide", first, first, shore),
        *group("tight", list(garden(tight)), first, whole(shore + creeks)),
        *bottom(whole(shore + creeks)),
        f'<path class="river" d="{outline(river, 62, -1e4, 26, 1.5)}"/>\n</svg>\n',
    ]
    assets = Path(__file__).parent.with_name("assets")
    assets.joinpath("valley.svg").write_text("\n".join(drawing))
    x, y = np.meshgrid(np.arange(WEST, EAST, CELL) + CELL / 2, np.arange(NORTH, SOUTH, CELL) + CELL / 2)
    chart = np.clip(sample(under, (x - XS[0]) / CELL, (y - YS[0]) / CELL) * 170 + 25.5, 0, 255).astype(np.uint8)
    assets.joinpath("depth.pgm").write_bytes(b"P5 %d %d 255\n" % chart.shape[::-1] + chart.tobytes())
