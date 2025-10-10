from dataclasses import asdict
from pathlib import Path
from typing import Any

from PIL import Image
from ruurd_photos_ml import get_captioner, get_facial_recognition, get_object_detection, get_ocr

captioner_instance = get_captioner()
facial_recognition_instance = get_facial_recognition()
object_detection_instance = get_object_detection()
ocr_instance = get_ocr()


def caption(image_path: Path, instruction: str | None = None) -> str:
    loaded_image = Image.open(image_path)
    return captioner_instance.caption(loaded_image, instruction)


def facial_recognition(image_path: Path) -> list[dict[str, Any]]:
    loaded_image = Image.open(image_path)
    return [asdict(x) for x in facial_recognition_instance.get_faces(loaded_image)]


def object_detection(image_path: Path) -> list[dict[str, Any]]:
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
