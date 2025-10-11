from dataclasses import asdict
from pathlib import Path
from typing import Any

import numpy as np
from PIL import Image
from numpy.typing import NDArray
from ruurd_photos_ml import (
    get_captioner,
    get_facial_recognition,
    get_object_detection,
    get_ocr,
    get_embedder,
)

captioner_instance = get_captioner()
facial_recognition_instance = get_facial_recognition()
object_detection_instance = get_object_detection()
ocr_instance = get_ocr()
embedder_instance = get_embedder()


def embed_image(image_path: Path) -> NDArray[np.float32]:
    loaded_image = Image.open(image_path)
    return embedder_instance.embed_image(loaded_image)


def embed_images(image_paths: list[Path]) -> NDArray[np.float32]:
    loaded_images = [Image.open(image_path) for image_path in image_paths]
    return embedder_instance.embed_images(loaded_images)


def embed_text(text: str) -> NDArray[np.float32]:
    return embedder_instance.embed_text(text)


def embed_texts(texts: list[str]) -> NDArray[np.float32]:
    return embedder_instance.embed_texts(texts)


def caption(image_path: Path, instruction: str | None = None) -> str:
    loaded_image = Image.open(image_path)
    return captioner_instance.caption(loaded_image, instruction)


def recognize_faces(image_path: Path) -> list[dict[str, Any]]:
    loaded_image = Image.open(image_path)
    return [asdict(x) for x in facial_recognition_instance.get_faces(loaded_image)]


def detect_objects(image_path: Path) -> list[dict[str, Any]]:
    loaded_image = Image.open(image_path)
    return [asdict(x) for x in object_detection_instance.detect_objects(loaded_image)]


def ocr(image_path: Path, languages: tuple[str, ...]) -> dict[str, Any]:
    loaded_image = Image.open(image_path)
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