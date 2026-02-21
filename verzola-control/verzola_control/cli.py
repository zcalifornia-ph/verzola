from __future__ import annotations

import argparse
from pathlib import Path
import sys
from typing import Sequence

from .validate.engine import PolicyValidationError, validate_policy_file


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="verzolactl",
        description="VERZOLA control-plane utility.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    validate_parser = subparsers.add_parser(
        "validate",
        help="Validate a VERZOLA policy file against the strict schema.",
    )
    validate_parser.add_argument(
        "policy_file",
        help="Path to policy config file (.yaml/.yml/.toml).",
    )
    validate_parser.add_argument(
        "--no-strict",
        action="store_true",
        help="Disable unknown-field rejection.",
    )

    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(list(argv) if argv is not None else None)

    if args.command == "validate":
        return _run_validate(policy_file=args.policy_file, strict=not args.no_strict)

    parser.print_help(sys.stderr)
    return 2


def _run_validate(*, policy_file: str, strict: bool) -> int:
    path = Path(policy_file)
    try:
        validate_policy_file(path, strict=strict)
    except FileNotFoundError:
        print(
            f"{path} [<io>] policy file does not exist",
            file=sys.stderr,
        )
        return 1
    except PolicyValidationError as error:
        for diagnostic in error.diagnostics:
            print(diagnostic.render(), file=sys.stderr)
        return 1

    mode = "strict" if strict else "non-strict"
    print(f"{path} validated successfully ({mode} mode).")
    return 0

