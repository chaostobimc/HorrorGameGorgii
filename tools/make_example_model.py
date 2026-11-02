#!/usr/bin/env python3
"""Builds `assets/models/room_example.glb`: a hollow room with a doorway.

It exists so the model import can be tested without Blender: drop it into the
hotel in the editor and walk inside — the collision follows the walls, the
inside stays empty.

The file is a plain glTF 2.0 binary (JSON chunk + BIN chunk), written by hand:
one mesh, proper per-face normals, no node transforms, unit = metre.
"""

import json
import os
import struct

# name, center, size -------------------------------------------------------
BOXES = [
    ("floor", (0.0, -0.1, 0.0), (6.4, 0.2, 6.4)),
    ("ceiling", (0.0, 3.1, 0.0), (6.4, 0.2, 6.4)),
    ("wall_north", (0.0, 1.5, -3.1), (6.4, 3.0, 0.2)),
    ("wall_south", (0.0, 1.5, 3.1), (6.4, 3.0, 0.2)),
    ("wall_west", (-3.1, 1.5, 0.0), (0.2, 3.0, 6.4)),
    # east wall split in two so the doorway stays open
    ("wall_east_a", (3.1, 1.5, -1.9), (0.2, 3.0, 2.6)),
    ("wall_east_b", (3.1, 1.5, 1.9), (0.2, 3.0, 2.6)),
    ("lintel", (3.1, 2.7, 0.0), (0.2, 0.6, 1.2)),
]

# faces: normal, four corners as signs of (x, y, z) -----------------------
FACES = [
    ((0.0, 0.0, 1.0), [(-1, -1, 1), (1, -1, 1), (1, 1, 1), (-1, 1, 1)]),
    ((0.0, 0.0, -1.0), [(1, -1, -1), (-1, -1, -1), (-1, 1, -1), (1, 1, -1)]),
    ((1.0, 0.0, 0.0), [(1, -1, 1), (1, -1, -1), (1, 1, -1), (1, 1, 1)]),
    ((-1.0, 0.0, 0.0), [(-1, -1, -1), (-1, -1, 1), (-1, 1, 1), (-1, 1, -1)]),
    ((0.0, 1.0, 0.0), [(-1, 1, 1), (1, 1, 1), (1, 1, -1), (-1, 1, -1)]),
    ((0.0, -1.0, 0.0), [(-1, -1, -1), (1, -1, -1), (1, -1, 1), (-1, -1, 1)]),
]

positions = []
normals = []
indices = []

for _name, center, size in BOXES:
    half = (size[0] * 0.5, size[1] * 0.5, size[2] * 0.5)
    for normal, corners in FACES:
        base = len(positions)
        for sx, sy, sz in corners:
            positions.append(
                (
                    center[0] + sx * half[0],
                    center[1] + sy * half[1],
                    center[2] + sz * half[2],
                )
            )
            normals.append(normal)
        indices.extend([base, base + 1, base + 2, base, base + 2, base + 3])

vertex_data = b""
for p, n in zip(positions, normals):
    vertex_data += struct.pack("<3f", *p)
    vertex_data += struct.pack("<3f", *n)
index_data = b"".join(struct.pack("<I", i) for i in indices)

position_bytes = len(positions) * 12
normal_bytes = len(normals) * 12
# interleave: position then normal per vertex
stride = 24
interleaved = b""
for p, n in zip(positions, normals):
    interleaved += struct.pack("<3f", *p) + struct.pack("<3f", *n)

offset_positions = 0
offset_normals = 12
bin_data = interleaved + index_data
offset_indices = len(interleaved)

min_v = [min(p[i] for p in positions) for i in range(3)]
max_v = [max(p[i] for p in positions) for i in range(3)]

gltf = {
    "asset": {"version": "2.0", "generator": "Diplomat Peak level tools"},
    "scene": 0,
    "scenes": [{"name": "Room", "nodes": [0]}],
    "nodes": [{"name": "Room", "mesh": 0}],
    "materials": [
        {
            "name": "plaster",
            "doubleSided": False,
            "pbrMetallicRoughness": {
                "baseColorFactor": [0.62, 0.60, 0.55, 1.0],
                "metallicFactor": 0.0,
                "roughnessFactor": 0.9,
            },
        }
    ],
    "meshes": [
        {
            "name": "Room",
            "primitives": [
                {
                    "attributes": {"POSITION": 0, "NORMAL": 1},
                    "indices": 2,
                    "material": 0,
                }
            ],
        }
    ],
    "accessors": [
        {
            "bufferView": 0,
            "byteOffset": offset_positions,
            "componentType": 5126,
            "count": len(positions),
            "type": "VEC3",
            "min": min_v,
            "max": max_v,
        },
        {
            "bufferView": 0,
            "byteOffset": offset_normals,
            "componentType": 5126,
            "count": len(normals),
            "type": "VEC3",
        },
        {
            "bufferView": 1,
            "byteOffset": 0,
            "componentType": 5125,
            "count": len(indices),
            "type": "SCALAR",
        },
    ],
    "bufferViews": [
        {
            "buffer": 0,
            "byteOffset": 0,
            "byteLength": len(interleaved),
            "byteStride": stride,
            "target": 34962,
        },
        {
            "buffer": 0,
            "byteOffset": offset_indices,
            "byteLength": len(index_data),
            "target": 34963,
        },
    ],
    "buffers": [{"byteLength": len(bin_data)}],
}

json_chunk = json.dumps(gltf, separators=(",", ":")).encode("utf-8")
json_chunk += b" " * ((4 - len(json_chunk) % 4) % 4)
bin_chunk = bin_data + b"\x00" * ((4 - len(bin_data) % 4) % 4)

total = 12 + 8 + len(json_chunk) + 8 + len(bin_chunk)
out = bytearray()
out += struct.pack("<III", 0x46546C67, 2, total)
out += struct.pack("<II", len(json_chunk), 0x4E4F534A)
out += json_chunk
out += struct.pack("<II", len(bin_chunk), 0x004E4942)
out += bin_chunk

target = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
                      "assets", "models", "room_example.glb")
os.makedirs(os.path.dirname(target), exist_ok=True)
with open(target, "wb") as handle:
    handle.write(bytes(out))
print(f"wrote {target} ({len(out)} bytes, {len(positions)} vertices, {len(indices) // 3} triangles)")
