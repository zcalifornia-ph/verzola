from __future__ import annotations

import argparse
from pathlib import Path
import sys
from typing import Sequence

from .render.engine import SUPPORTED_ENVIRONMENTS, render_policy_file
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

    render_parser = subparsers.add_parser(
        "render",
        help="Render deterministic effective proxy config from a policy file.",
    )
    render_parser.add_argument(
        "policy_file",
        help="Path to policy config file (.yaml/.yml/.toml).",
    )
    render_parser.add_argument(
        "--environment",
        default="dev",
        choices=SUPPORTED_ENVIRONMENTS,
        help="Deployment environment profile for rendered output.",
    )
    render_parser.add_argument(
        "--output",
        default="-",
        help="Output path for rendered JSON ('-' prints to stdout).",
    )
    render_parser.add_argument(
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
    if args.command == "render":
        return _run_render(
            policy_file=args.policy_file,
            environment=args.environment,
            output_path=args.output,
            strict=not args.no_strict,
        )

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


def _run_render(
    *,
    policy_file: str,
    environment: str,
    output_path: str,
    strict: bool,
) -> int:
    path = Path(policy_file)
    try:
        rendered = render_policy_file(path, environment=environment, strict=strict)
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
    except ValueError as error:
        print(f"{path} [<render>] {error}", file=sys.stderr)
        return 1

    if output_path == "-":
        print(rendered)
        return 0

    destination = Path(output_path)
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(rendered + "\n", encoding="utf-8")
    mode = "strict" if strict else "non-strict"
    print(
        f"{path} rendered successfully for environment '{environment}' "
        f"({mode} mode) -> {destination}"
    )
    return 0
