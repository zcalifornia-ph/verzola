from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import re
from typing import Any
import tomllib


class PolicyParseError(ValueError):
    def __init__(
        self,
        *,
        file_path: str,
        field_path: str,
        message: str,
        suggestion: str | None = None,
    ) -> None:
        super().__init__(message)
        self.file_path = file_path
        self.field_path = field_path
        self.message = message
        self.suggestion = suggestion


def parse_policy_file(path: str | Path) -> dict[str, Any]:
    policy_path = Path(path)
    text = policy_path.read_text(encoding="utf-8")
    return parse_policy_text(text, file_path=str(policy_path))


def parse_policy_text(text: str, *, file_path: str) -> dict[str, Any]:
    extension = Path(file_path).suffix.lower()
    if extension == ".toml":
        return _parse_toml(text, file_path=file_path)
    if extension in {".yaml", ".yml"}:
        return _parse_simple_yaml(text, file_path=file_path)

    raise PolicyParseError(
        file_path=file_path,
        field_path="<parse>",
        message=f"unsupported policy file extension '{extension or '<none>'}'",
        suggestion="Use .yaml, .yml, or .toml for policy files.",
    )


def _parse_toml(text: str, *, file_path: str) -> dict[str, Any]:
    try:
        parsed = tomllib.loads(text)
    except tomllib.TOMLDecodeError as error:
        raise PolicyParseError(
            file_path=file_path,
            field_path="<parse>",
            message=f"invalid TOML syntax: {error}",
            suggestion="Fix TOML syntax at the reported line/column.",
        ) from error

    if not isinstance(parsed, dict):
        raise PolicyParseError(
            file_path=file_path,
            field_path="<root>",
            message="policy file must deserialize to a top-level object",
            suggestion="Use key/value tables at the top level.",
        )

    return parsed


@dataclass
class _YamlContext:
    indent: int
    mapping: dict[str, Any]
    path: str


def _parse_simple_yaml(text: str, *, file_path: str) -> dict[str, Any]:
    root: dict[str, Any] = {}
    stack: list[_YamlContext] = [_YamlContext(indent=-1, mapping=root, path="")]

    for line_number, raw_line in enumerate(text.splitlines(), start=1):
        line = raw_line.rstrip("\r\n")
        if not line.strip():
            continue
        if line.lstrip(" ").startswith("#"):
            continue

        leading = line[: len(line) - len(line.lstrip(" \t"))]
        if "\t" in leading:
            raise _yaml_parse_error(
                file_path=file_path,
                line_number=line_number,
                message="tab indentation is not supported",
                suggestion="Use spaces with 2-space indentation.",
            )

        indent = len(line) - len(line.lstrip(" "))
        if indent % 2 != 0:
            raise _yaml_parse_error(
                file_path=file_path,
                line_number=line_number,
                message="indentation must use 2-space steps",
                suggestion="Align nested keys to multiples of 2 spaces.",
            )

        content = _strip_inline_yaml_comment(line[indent:]).strip()
        if not content:
            continue

        if ":" not in content:
            raise _yaml_parse_error(
                file_path=file_path,
                line_number=line_number,
                message="expected 'key: value' syntax",
                suggestion="Add ':' between the key and value.",
            )

        key_part, value_part = content.split(":", 1)
        key = key_part.strip()
        value = value_part.strip()
        if not key:
            raise _yaml_parse_error(
                file_path=file_path,
                line_number=line_number,
                message="key cannot be empty",
                suggestion="Provide a non-empty key before ':'.",
            )

        while len(stack) > 1 and indent <= stack[-1].indent:
            stack.pop()

        parent = stack[-1]
        if indent > parent.indent + 2:
            raise _yaml_parse_error(
                file_path=file_path,
                line_number=line_number,
                message="indentation jumps over expected nesting level",
                suggestion="Nest child keys one level (2 spaces) deeper than parent.",
            )

        if key in parent.mapping:
            field_path = f"{parent.path}.{key}" if parent.path else key
            raise PolicyParseError(
                file_path=file_path,
                field_path=field_path,
                message="duplicate key in YAML mapping",
                suggestion="Remove or rename one of the duplicate keys.",
            )

        field_path = f"{parent.path}.{key}" if parent.path else key
        if value == "":
            child: dict[str, Any] = {}
            parent.mapping[key] = child
            stack.append(_YamlContext(indent=indent, mapping=child, path=field_path))
            continue

        parent.mapping[key] = _parse_yaml_scalar(
            value=value,
            file_path=file_path,
            line_number=line_number,
            field_path=field_path,
        )

    return root


def _parse_yaml_scalar(
    *,
    value: str,
    file_path: str,
    line_number: int,
    field_path: str,
) -> Any:
    lowered = value.lower()
    if lowered == "true":
        return True
    if lowered == "false":
        return False
    if lowered in {"null", "~"}:
        return None
    if re.fullmatch(r"[+-]?[0-9]+", value):
        return int(value)
    if (
        len(value) >= 2
        and value[0] == value[-1]
        and value[0] in {"'", '"'}
    ):
        return value[1:-1]
    if value[0] in {"'", '"'} and value[-1] != value[0]:
        raise _yaml_parse_error(
            file_path=file_path,
            line_number=line_number,
            message="unclosed quoted scalar",
            suggestion="Close the quoted string with a matching quote.",
        )
    return value


def _strip_inline_yaml_comment(content: str) -> str:
    in_single = False
    in_double = False

    for index, character in enumerate(content):
        if character == "'" and not in_double:
            in_single = not in_single
        elif character == '"' and not in_single:
            in_double = not in_double
        elif (
            character == "#"
            and not in_single
            and not in_double
            and (index == 0 or content[index - 1].isspace())
        ):
            return content[:index].rstrip()

    return content


def _yaml_parse_error(
    *,
    file_path: str,
    line_number: int,
    message: str,
    suggestion: str | None = None,
) -> PolicyParseError:
    return PolicyParseError(
        file_path=file_path,
        field_path=f"<parse>:line:{line_number}",
        message=f"invalid YAML syntax at line {line_number}: {message}",
        suggestion=suggestion,
    )

