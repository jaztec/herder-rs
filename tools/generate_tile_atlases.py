#!/usr/bin/env python3
"""Generate terrain autotile atlases from assets/textures/backgrounds.png.

The generated atlases use a 4-bit cardinal-neighbor mask:

    north = 1
    east  = 2
    south = 4
    west  = 8

Atlas index equals the mask value. The files are laid out as a 4x4 grid of
150x150 tiles, matching the original backgrounds.png tile size.
"""

from __future__ import annotations

import math
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "assets" / "textures" / "backgrounds.png"
WATER_OUT = ROOT / "assets" / "textures" / "water_autotile.png"
FLOWERS_OUT = ROOT / "assets" / "textures" / "flowers_autotile.png"
PATH_OUT = ROOT / "assets" / "textures" / "path_autotile.png"

TILE_SIZE = 150
SOURCE_COLUMNS = 3
ATLAS_COLUMNS = 4
ATLAS_ROWS = 4

NORTH = 1
EAST = 2
SOUTH = 4
WEST = 8


def identify(path: Path) -> tuple[int, int]:
    output = subprocess.check_output(
        ["magick", "identify", "-format", "%w %h", str(path)],
        text=True,
    )
    width, height = output.strip().split()
    return int(width), int(height)


def load_rgba(path: Path) -> tuple[int, int, bytearray]:
    width, height = identify(path)
    data = subprocess.check_output(
        ["magick", str(path), "-depth", "8", "rgba:-"],
    )
    return width, height, bytearray(data)


def save_rgba(path: Path, width: int, height: int, data: bytearray) -> None:
    subprocess.run(
        ["magick", "-size", f"{width}x{height}", "-depth", "8", "rgba:-", str(path)],
        input=bytes(data),
        check=True,
    )


def pixel(data: bytearray, width: int, x: int, y: int) -> tuple[int, int, int, int]:
    offset = (y * width + x) * 4
    return tuple(data[offset : offset + 4])


def set_pixel(
    data: bytearray,
    width: int,
    x: int,
    y: int,
    color: tuple[int, int, int, int],
) -> None:
    offset = (y * width + x) * 4
    data[offset : offset + 4] = bytes(color)


def crop_tile(
    data: bytearray,
    width: int,
    tile_index: int,
) -> bytearray:
    tile_x = tile_index % SOURCE_COLUMNS
    tile_y = tile_index // SOURCE_COLUMNS
    output = bytearray(TILE_SIZE * TILE_SIZE * 4)

    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            color = pixel(
                data,
                width,
                tile_x * TILE_SIZE + x,
                tile_y * TILE_SIZE + y,
            )
            set_pixel(output, TILE_SIZE, x, y, color)

    return output


def paste_tile(
    atlas: bytearray,
    atlas_width: int,
    tile: bytearray,
    tile_index: int,
) -> None:
    offset_x = (tile_index % ATLAS_COLUMNS) * TILE_SIZE
    offset_y = (tile_index // ATLAS_COLUMNS) * TILE_SIZE

    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            set_pixel(
                atlas,
                atlas_width,
                offset_x + x,
                offset_y + y,
                pixel(tile, TILE_SIZE, x, y),
            )


def blend(
    left: tuple[int, int, int, int],
    right: tuple[int, int, int, int],
    amount: float,
) -> tuple[int, int, int, int]:
    return (
        int(left[0] * (1.0 - amount) + right[0] * amount),
        int(left[1] * (1.0 - amount) + right[1] * amount),
        int(left[2] * (1.0 - amount) + right[2] * amount),
        255,
    )


def color_distance(left: tuple[int, int, int, int], right: tuple[int, int, int, int]) -> int:
    return abs(left[0] - right[0]) + abs(left[1] - right[1]) + abs(left[2] - right[2])


def average_color(colors: list[tuple[int, int, int, int]]) -> tuple[int, int, int, int]:
    if not colors:
        return (0, 0, 0, 255)

    return (
        sum(color[0] for color in colors) // len(colors),
        sum(color[1] for color in colors) // len(colors),
        sum(color[2] for color in colors) // len(colors),
        255,
    )


def shade(color: tuple[int, int, int, int], amount: int) -> tuple[int, int, int, int]:
    return (
        max(0, min(255, color[0] + amount)),
        max(0, min(255, color[1] + amount)),
        max(0, min(255, color[2] + amount)),
        255,
    )


def is_water_pixel(color: tuple[int, int, int, int]) -> bool:
    return color[2] > color[0] + 25 and color[2] > color[1] + 15 and color[2] > 95


def is_flower_pixel(
    source: tuple[int, int, int, int],
    grass: tuple[int, int, int, int],
) -> bool:
    red, green, blue, _ = source
    colorful = red > 145 and blue > 115 and abs(red - green) > 25
    yellow = red > 175 and green > 150 and blue < 125
    white = red > 190 and green > 190 and blue > 180 and color_distance(source, grass) > 65
    return colorful or yellow or white


def water_mask(mask: int) -> list[bool]:
    values: list[bool] = []
    connection_count = (
        int(bool(mask & NORTH))
        + int(bool(mask & EAST))
        + int(bool(mask & SOUTH))
        + int(bool(mask & WEST))
    )

    for y in range(TILE_SIZE):
        dy = (y + 0.5 - TILE_SIZE / 2) / (TILE_SIZE / 2)
        for x in range(TILE_SIZE):
            dx = (x + 0.5 - TILE_SIZE / 2) / (TILE_SIZE / 2)

            if connection_count == 4:
                inside = True
            else:
                inside = (dx / 0.68) ** 2 + (dy / 0.62) ** 2 <= 1.0

            if mask & NORTH:
                inside = inside or y < TILE_SIZE * 0.56
            if mask & EAST:
                inside = inside or x > TILE_SIZE * 0.44
            if mask & SOUTH:
                inside = inside or y > TILE_SIZE * 0.44
            if mask & WEST:
                inside = inside or x < TILE_SIZE * 0.56

            if (mask & NORTH) and (mask & EAST):
                inside = inside or (x > TILE_SIZE * 0.36 and y < TILE_SIZE * 0.64)
            if (mask & EAST) and (mask & SOUTH):
                inside = inside or (x > TILE_SIZE * 0.36 and y > TILE_SIZE * 0.36)
            if (mask & SOUTH) and (mask & WEST):
                inside = inside or (x < TILE_SIZE * 0.64 and y > TILE_SIZE * 0.36)
            if (mask & WEST) and (mask & NORTH):
                inside = inside or (x < TILE_SIZE * 0.64 and y < TILE_SIZE * 0.64)

            noise = math.sin(x * 0.21 + y * 0.13) * 0.025
            edge_taper = 0.97 + noise
            connected_edge = (
                (mask & NORTH and y < 8)
                or (mask & EAST and x >= TILE_SIZE - 8)
                or (mask & SOUTH and y >= TILE_SIZE - 8)
                or (mask & WEST and x < 8)
            )

            if connection_count == 4:
                shore_noise = 0.04 * math.sin(x * 0.17) + 0.03 * math.sin(y * 0.23)
                inside = abs(dx) < 0.98 + shore_noise and abs(dy) < 0.98 - shore_noise

            if connected_edge:
                values.append(True)
            else:
                values.append(inside and abs(dx) < edge_taper and abs(dy) < edge_taper)

    return values


def dilate(mask: list[bool], radius: int) -> list[bool]:
    current = mask[:]
    for _ in range(radius):
        expanded = current[:]
        for y in range(TILE_SIZE):
            for x in range(TILE_SIZE):
                index = y * TILE_SIZE + x
                if current[index]:
                    continue
                expanded[index] = (
                    (x > 0 and current[index - 1])
                    or (x < TILE_SIZE - 1 and current[index + 1])
                    or (y > 0 and current[index - TILE_SIZE])
                    or (y < TILE_SIZE - 1 and current[index + TILE_SIZE])
                )
        current = expanded

    return current


def build_water_variant(
    grass: bytearray,
    water_source: bytearray,
    mask: int,
) -> bytearray:
    output = bytearray(grass)

    water_samples = [
        pixel(water_source, TILE_SIZE, x, y)
        for y in range(TILE_SIZE)
        for x in range(TILE_SIZE)
        if is_water_pixel(pixel(water_source, TILE_SIZE, x, y))
    ]
    bank_samples = [
        pixel(water_source, TILE_SIZE, x, y)
        for y in range(TILE_SIZE)
        for x in range(TILE_SIZE)
        if not is_water_pixel(pixel(water_source, TILE_SIZE, x, y))
        and color_distance(
            pixel(water_source, TILE_SIZE, x, y),
            pixel(grass, TILE_SIZE, x, y),
        )
        > 80
    ]

    water_average = average_color(water_samples) or (70, 145, 195, 255)
    bank_average = average_color(bank_samples) or (115, 80, 36, 255)

    shape = water_mask(mask)
    bank = dilate(shape, 13)

    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            index = y * TILE_SIZE + x
            grass_color = pixel(grass, TILE_SIZE, x, y)

            if shape[index]:
                sample = water_samples[(x * 19 + y * 11) % len(water_samples)]
                ripple = int(
                    8 * math.sin((x + y) * 0.09)
                    + 5 * math.sin(x * 0.17)
                    + 4 * math.sin(y * 0.13)
                )
                color = blend(shade(water_average, ripple), sample, 0.18)
                set_pixel(output, TILE_SIZE, x, y, color)
            elif bank[index]:
                sample = bank_samples[(x * 7 + y * 17) % len(bank_samples)]
                dirt = blend(bank_average, sample, 0.55)
                set_pixel(output, TILE_SIZE, x, y, blend(grass_color, dirt, 0.82))

    return output


def path_mask(mask: int) -> list[bool]:
    values: list[bool] = []
    half_width = TILE_SIZE * 0.19
    endpoint_radius = TILE_SIZE * 0.29
    center = (TILE_SIZE - 1) / 2

    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            dx = x - center
            dy = y - center

            inside = dx * dx + dy * dy <= endpoint_radius * endpoint_radius

            if mask & NORTH:
                inside = inside or (abs(dx) <= half_width and y <= center)
            if mask & EAST:
                inside = inside or (abs(dy) <= half_width and x >= center)
            if mask & SOUTH:
                inside = inside or (abs(dx) <= half_width and y >= center)
            if mask & WEST:
                inside = inside or (abs(dy) <= half_width and x <= center)

            values.append(inside)

    return values


def build_path_variant(
    grass: bytearray,
    water_source: bytearray,
    mask: int,
) -> bytearray:
    output = bytearray(grass)
    dirt_samples = [
        pixel(water_source, TILE_SIZE, x, y)
        for y in range(TILE_SIZE)
        for x in range(TILE_SIZE)
        if not is_water_pixel(pixel(water_source, TILE_SIZE, x, y))
        and color_distance(
            pixel(water_source, TILE_SIZE, x, y),
            pixel(grass, TILE_SIZE, x, y),
        )
        > 70
    ]
    dirt_average = average_color(dirt_samples) or (118, 82, 38, 255)

    shape = path_mask(mask)
    edge = dilate(shape, 5)

    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            index = y * TILE_SIZE + x
            grass_color = pixel(grass, TILE_SIZE, x, y)

            if shape[index]:
                sample = dirt_samples[(x * 11 + y * 29 + mask * 7) % len(dirt_samples)]
                noise = int(9 * math.sin(x * 0.31) + 6 * math.sin(y * 0.23))
                dirt = blend(shade(dirt_average, noise), sample, 0.38)
                set_pixel(output, TILE_SIZE, x, y, blend(grass_color, dirt, 0.9))
            elif edge[index]:
                sample = dirt_samples[(x * 17 + y * 5 + mask * 13) % len(dirt_samples)]
                dirt = blend(dirt_average, sample, 0.4)
                set_pixel(output, TILE_SIZE, x, y, blend(grass_color, dirt, 0.45))

    return output


def hash01(x: int, y: int, seed: int) -> float:
    value = (x * 374761393 + y * 668265263 + seed * 1442695041) & 0xFFFFFFFF
    value = (value ^ (value >> 13)) * 1274126177 & 0xFFFFFFFF
    return ((value ^ (value >> 16)) & 0xFFFFFF) / float(0xFFFFFF)


def flower_samples(grass: bytearray, flowers: bytearray) -> list[tuple[int, int, int, int]]:
    return [
        pixel(flowers, TILE_SIZE, x, y)
        for y in range(TILE_SIZE)
        for x in range(TILE_SIZE)
        if is_flower_pixel(pixel(flowers, TILE_SIZE, x, y), pixel(grass, TILE_SIZE, x, y))
    ]


def flower_density(mask: int, x: int, y: int) -> float:
    dx = (x + 0.5 - TILE_SIZE / 2) / (TILE_SIZE / 2)
    dy = (y + 0.5 - TILE_SIZE / 2) / (TILE_SIZE / 2)
    density = max(0.0, 1.0 - ((dx / 0.58) ** 2 + (dy / 0.48) ** 2)) * 0.82

    if mask & NORTH:
        density = max(density, max(0.0, 1.0 - y / 58.0) * 0.68)
    if mask & EAST:
        density = max(density, max(0.0, 1.0 - (TILE_SIZE - 1 - x) / 58.0) * 0.68)
    if mask & SOUTH:
        density = max(density, max(0.0, 1.0 - (TILE_SIZE - 1 - y) / 58.0) * 0.68)
    if mask & WEST:
        density = max(density, max(0.0, 1.0 - x / 58.0) * 0.68)

    if (mask & NORTH) and (mask & EAST):
        density = max(density, max(0.0, 1.0 - (x - y + 35) / 120.0) * 0.45)
    if (mask & EAST) and (mask & SOUTH):
        density = max(density, max(0.0, 1.0 - ((TILE_SIZE - 1 - x) + (TILE_SIZE - 1 - y)) / 125.0) * 0.55)
    if (mask & SOUTH) and (mask & WEST):
        density = max(density, max(0.0, 1.0 - (x + (TILE_SIZE - 1 - y)) / 125.0) * 0.55)
    if (mask & WEST) and (mask & NORTH):
        density = max(density, max(0.0, 1.0 - (x + y) / 125.0) * 0.55)

    return density


def draw_flower_dot(
    output: bytearray,
    x: int,
    y: int,
    color: tuple[int, int, int, int],
) -> None:
    for dy in range(-2, 3):
        for dx in range(-2, 3):
            target_x = x + dx
            target_y = y + dy
            if target_x < 0 or target_x >= TILE_SIZE or target_y < 0 or target_y >= TILE_SIZE:
                continue

            distance = abs(dx) + abs(dy)
            amount = 0.95 if distance == 0 else 0.7 if distance <= 2 else 0.42
            existing = pixel(output, TILE_SIZE, target_x, target_y)
            set_pixel(output, TILE_SIZE, target_x, target_y, blend(existing, color, amount))


def build_flower_variant(grass: bytearray, flowers: bytearray, mask: int) -> bytearray:
    output = bytearray(grass)
    samples = flower_samples(grass, flowers)

    for y in range(1, TILE_SIZE, 3):
        for x in range(1, TILE_SIZE, 3):
            density = flower_density(mask, x, y)
            if density <= 0.0:
                continue

            chance = 0.48 * density
            if hash01(x, y, mask) > chance:
                continue

            jitter_x = int(hash01(x, y, mask + 17) * 5) - 2
            jitter_y = int(hash01(x, y, mask + 31) * 5) - 2
            color = samples[(x * 13 + y * 7 + mask * 19) % len(samples)]
            draw_flower_dot(output, x + jitter_x, y + jitter_y, color)

            if hash01(x, y, mask + 53) < 0.38:
                side_color = samples[(x * 5 + y * 23 + mask * 11) % len(samples)]
                side_x = x + jitter_x + int(hash01(x, y, mask + 71) * 11) - 5
                side_y = y + jitter_y + int(hash01(x, y, mask + 89) * 11) - 5
                draw_flower_dot(output, side_x, side_y, side_color)

    return output


def build_atlases() -> None:
    width, _height, source = load_rgba(SOURCE)
    grass = crop_tile(source, width, 0)
    water = crop_tile(source, width, 1)
    flowers = crop_tile(source, width, 2)

    atlas_width = ATLAS_COLUMNS * TILE_SIZE
    atlas_height = ATLAS_ROWS * TILE_SIZE
    water_atlas = bytearray(atlas_width * atlas_height * 4)
    flower_atlas = bytearray(atlas_width * atlas_height * 4)
    path_atlas = bytearray(atlas_width * atlas_height * 4)

    for mask in range(16):
        paste_tile(water_atlas, atlas_width, build_water_variant(grass, water, mask), mask)
        paste_tile(flower_atlas, atlas_width, build_flower_variant(grass, flowers, mask), mask)
        paste_tile(path_atlas, atlas_width, build_path_variant(grass, water, mask), mask)

    save_rgba(WATER_OUT, atlas_width, atlas_height, water_atlas)
    save_rgba(FLOWERS_OUT, atlas_width, atlas_height, flower_atlas)
    save_rgba(PATH_OUT, atlas_width, atlas_height, path_atlas)
    print(f"Wrote {WATER_OUT.relative_to(ROOT)}")
    print(f"Wrote {FLOWERS_OUT.relative_to(ROOT)}")
    print(f"Wrote {PATH_OUT.relative_to(ROOT)}")


if __name__ == "__main__":
    build_atlases()
