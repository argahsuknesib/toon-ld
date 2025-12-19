"""Type stubs for toon_ld module."""

from typing import Any, Dict, List, Union

__version__: str

def convert_jsonld_to_toon(json_str: str) -> str:
    """Convert JSON-LD string to TOON-LD format.

    Args:
        json_str: A JSON or JSON-LD formatted string.

    Returns:
        TOON-LD formatted string.

    Raises:
        ValueError: If the input is not valid JSON.
    """
    ...

def convert_toon_to_jsonld(toon_str: str) -> str:
    """Convert TOON-LD string to JSON-LD format.

    Args:
        toon_str: A TOON-LD formatted string.

    Returns:
        JSON-LD formatted string (pretty-printed).

    Raises:
        ValueError: If the input is not valid TOON-LD.
    """
    ...

def validate_toon(toon_str: str) -> bool:
    """Validate a TOON-LD string.

    Args:
        toon_str: A TOON-LD formatted string.

    Returns:
        True if the string is valid TOON-LD, False otherwise.
    """
    ...

def validate_json(json_str: str) -> bool:
    """Validate a JSON string.

    Args:
        json_str: A JSON formatted string.

    Returns:
        True if the string is valid JSON, False otherwise.
    """
    ...

def parse_toon(toon_str: str) -> Dict[str, Any]:
    """Parse TOON-LD string to a Python dictionary.

    Args:
        toon_str: A TOON-LD formatted string.

    Returns:
        Python dictionary representing the parsed data.

    Raises:
        ValueError: If the input is not valid TOON-LD.
    """
    ...

def serialize_to_toon(
    data: Union[Dict[str, Any], List[Any], str, int, float, bool, None],
) -> str:
    """Serialize a Python dictionary to TOON-LD format.

    Args:
        data: A Python dictionary or JSON-serializable object.

    Returns:
        TOON-LD formatted string.

    Raises:
        ValueError: If the input cannot be serialized.
    """
    ...
