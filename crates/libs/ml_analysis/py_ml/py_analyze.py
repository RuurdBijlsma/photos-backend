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
    get_captioner,
    get_facial_recognition,
    get_object_detection,
    get_ocr,
    get_embedder, get_llm, ChatMessage,
)

captioner_instance = get_captioner()
facial_recognition_instance = get_facial_recognition()
object_detection_instance = get_object_detection()
ocr_instance = get_ocr()
embedder_instance = get_embedder()
llm_instance = get_llm()


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
    return embedder_instance.embed_image(load_image(image_path))


def embed_images(image_paths: list[Path]) -> NDArray[np.float32]:
    loaded_images = [load_image(image_path) for image_path in image_paths]
    return embedder_instance.embed_images(loaded_images)


def embed_text(text: str) -> NDArray[np.float32]:
    return embedder_instance.embed_text(text)


def embed_texts(texts: list[str]) -> NDArray[np.float32]:
    return embedder_instance.embed_texts(texts)


def caption(image_path: Path, instruction: str | None = None) -> str:
    return captioner_instance.caption(load_image(image_path), instruction)


def recognize_faces(image_path: Path) -> list[dict[str, Any]]:
    return [
        asdict(x) for x in facial_recognition_instance.get_faces(load_image(image_path))
    ]


def detect_objects(image_path: Path) -> list[dict[str, Any]]:
    return [
        asdict(x)
        for x in object_detection_instance.detect_objects(load_image(image_path))
    ]


def ocr(image_path: Path, languages: tuple[str, ...]) -> dict[str, Any]:
    loaded_image = load_image(image_path)
    has_text = ocr_instance.has_legible_text(loaded_image)
    if not has_text:
        return {
            "has_legible_text": has_text,
            "ocr_text": None,
            "ocr_boxes": None,
        }

    ocr_text = ocr_instance.get_text(loaded_image, languages)
    ocr_boxes = ocr_instance.get_boxes(loaded_image, languages)
    boxes_dicts = [asdict(x) for x in ocr_boxes]
    return {
        "has_legible_text": has_text,
        "ocr_text": ocr_text,
        "ocr_boxes": boxes_dicts,
    }


def llm_chat(messages: list[ChatMessage]) -> str:
    return llm_instance.chat(messages)
