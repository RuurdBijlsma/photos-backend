import os

os.environ["OMP_NUM_THREADS"] = "1"
os.environ["MKL_NUM_THREADS"] = "1"

import torch

torch.set_num_threads(1)

from dataclasses import asdict
from functools import lru_cache
from pathlib import Path
from typing import Any

import numpy as np
from PIL import Image
from material_color_utilities import (
    prominent_colors_from_image,
    theme_from_color,
    Variant,
)
from numpy.typing import NDArray
from ruurd_photos_ml import (
    get_facial_recognition,
    get_object_detection,
    get_embedder)


@lru_cache(maxsize=1)
def _get_facial_recognition():
    return get_facial_recognition()


@lru_cache(maxsize=1)
def _get_object_detection():
    return get_object_detection()


@lru_cache(maxsize=1)
def _get_embedder():
    return get_embedder()


@lru_cache(maxsize=256)
def load_image(path: Path) -> Image.Image:
    return Image.open(path)


def get_image_prominent_colors(image_path: Path) -> list[str]:
    loaded_image = load_image(image_path)
    return prominent_colors_from_image(loaded_image)[0:3]


def get_theme_from_color(
        color: str, variant: str, contrast_level: float
) -> dict[str, Any]:
    return theme_from_color(
        color,
        variant={
            "monochrome": Variant.MONOCHROME,
            "neutral": Variant.NEUTRAL,
            "tonalspot": Variant.TONALSPOT,
            "vibrant": Variant.VIBRANT,
            "expressive": Variant.EXPRESSIVE,
            "fidelity": Variant.FIDELITY,
            "content": Variant.CONTENT,
            "rainbow": Variant.RAINBOW,
            "fruitsalad": Variant.FRUITSALAD,
        }[variant.lower()],
        contrast_level=contrast_level,
    ).dict()


def embed_image(image_path: Path) -> NDArray[np.float32]:
    return _get_embedder().embed_image(load_image(image_path))


def embed_images(image_paths: list[Path]) -> NDArray[np.float32]:
    loaded_images = [load_image(image_path) for image_path in image_paths]
    return _get_embedder().embed_images(loaded_images)


def embed_text(text: str) -> NDArray[np.float32]:
    return _get_embedder().embed_text(text)


def embed_texts(texts: list[str]) -> NDArray[np.float32]:
    return _get_embedder().embed_texts(texts)


def recognize_faces(image_path: Path) -> list[dict[str, Any]]:
    return [
        asdict(x) for x in _get_facial_recognition().get_faces(load_image(image_path))
    ]


def detect_objects(image_path: Path) -> list[dict[str, Any]]:
    return [
        asdict(x)
        for x in _get_object_detection().detect_objects(load_image(image_path))
    ]
