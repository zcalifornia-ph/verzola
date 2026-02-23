from __future__ import annotations

from dataclasses import dataclass
from difflib import get_close_matches
from pathlib import Path
import re
from typing import Any, Mapping

from ..policy.model import (
    CapabilityHints,
    DnsTxtHint,
    DomainPolicy,
    ListenerPolicy,
    ListenerSet,
    MismatchAction,
    PolicyConfig,
    PolicyMode,
)
from ..policy.parser import PolicyParseError, parse_policy_file

DOMAIN_PATTERN = re.compile(
    r"(?=.{1,253}$)"
    r"[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?"
    r"(?:\.[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?)*"
)


@dataclass(frozen=True)
class Diagnostic:
    file_path: str
    field_path: str
    message: str
    suggestion: str | None = None

    def render(self) -> str:
        location = self.field_path or "<root>"
        if self.suggestion:
            return (
                f"{self.file_path} [{location}] {self.message} "
                f"Suggestion: {self.suggestion}"
            )
        return f"{self.file_path} [{location}] {self.message}"


class PolicyValidationError(ValueError):
    def __init__(self, diagnostics: list[Diagnostic]) -> None:
        super().__init__("policy validation failed")
        self.diagnostics = diagnostics

    def __str__(self) -> str:
        return "\n".join(diagnostic.render() for diagnostic in self.diagnostics)


class _Collector:
    def __init__(self, file_path: str) -> None:
        self.file_path = file_path
        self.diagnostics: list[Diagnostic] = []

    def add(
        self,
        *,
        field_path: str,
        message: str,
        suggestion: str | None = None,
    ) -> None:
        self.diagnostics.append(
            Diagnostic(
                file_path=self.file_path,
                field_path=field_path,
                message=message,
                suggestion=suggestion,
            )
        )

    def fail_if_any(self) -> None:
        if self.diagnostics:
            raise PolicyValidationError(self.diagnostics)


def validate_policy_file(path: str | Path, *, strict: bool = True) -> PolicyConfig:
    policy_path = Path(path)
    try:
        raw_data = parse_policy_file(policy_path)
    except PolicyParseError as parse_error:
        raise PolicyValidationError(
            [
                Diagnostic(
                    file_path=parse_error.file_path,
                    field_path=parse_error.field_path,
                    message=parse_error.message,
                    suggestion=parse_error.suggestion,
                )
            ]
        ) from parse_error

    return validate_policy_data(
        raw_data,
        file_path=str(policy_path),
        strict=strict,
    )


def validate_policy_data(
    raw_data: Mapping[str, Any],
    *,
    file_path: str,
    strict: bool = True,
) -> PolicyConfig:
    collector = _Collector(file_path=file_path)
    root = _expect_mapping(collector, raw_data, "<root>")
    if root is None:
        collector.fail_if_any()
        raise AssertionError("unreachable")

    required_top = {"version", "listeners"}
    allowed_top = {"version", "listeners", "domains", "capability_hints"}

    _require_keys(
        collector,
        container=root,
        required=required_top,
        field_path="<root>",
    )
    if strict:
        _reject_unknown_keys(
            collector,
            container=root,
            allowed=allowed_top,
            field_path="<root>",
        )

    version = _validate_version(collector, root.get("version"), "version")
    listeners = _validate_listeners(
        collector,
        root.get("listeners"),
        "listeners",
        strict=strict,
    )
    domains = _validate_domains(
        collector,
        root.get("domains", {}),
        "domains",
        strict=strict,
    )
    capability_hints = _validate_capability_hints(
        collector,
        root.get("capability_hints", {}),
        "capability_hints",
        strict=strict,
    )

    collector.fail_if_any()

    assert version is not None
    assert listeners is not None
    assert domains is not None
    assert capability_hints is not None

    return PolicyConfig(
        version=version,
        listeners=listeners,
        domains=domains,
        capability_hints=capability_hints,
    )


def _validate_version(
    collector: _Collector,
    raw_value: Any,
    field_path: str,
) -> int | None:
    if raw_value is None:
        collector.add(
            field_path=field_path,
            message="missing required field",
            suggestion="Set version: 1",
        )
        return None

    if not _is_int(raw_value):
        collector.add(
            field_path=field_path,
            message=f"expected integer version, got {type(raw_value).__name__}",
            suggestion="Set version to numeric value 1.",
        )
        return None

    if raw_value != 1:
        collector.add(
            field_path=field_path,
            message=f"unsupported schema version '{raw_value}'",
            suggestion="Use version: 1 for this release.",
        )
        return None

    return int(raw_value)


def _validate_listeners(
    collector: _Collector,
    raw_value: Any,
    field_path: str,
    *,
    strict: bool,
) -> ListenerSet | None:
    listeners_obj = _expect_mapping(collector, raw_value, field_path)
    if listeners_obj is None:
        return None

    required_keys = {"inbound", "outbound"}
    _require_keys(
        collector,
        container=listeners_obj,
        required=required_keys,
        field_path=field_path,
    )
    if strict:
        _reject_unknown_keys(
            collector,
            container=listeners_obj,
            allowed=required_keys,
            field_path=field_path,
        )

    inbound = _validate_listener_policy(
        collector,
        listeners_obj.get("inbound"),
        f"{field_path}.inbound",
        strict=strict,
    )
    outbound = _validate_listener_policy(
        collector,
        listeners_obj.get("outbound"),
        f"{field_path}.outbound",
        strict=strict,
    )

    if inbound is None or outbound is None:
        return None

    return ListenerSet(inbound=inbound, outbound=outbound)


def _validate_listener_policy(
    collector: _Collector,
    raw_value: Any,
    field_path: str,
    *,
    strict: bool,
) -> ListenerPolicy | None:
    listener_obj = _expect_mapping(collector, raw_value, field_path)
    if listener_obj is None:
        return None

    required_keys = {"mode", "allow_plaintext"}
    _require_keys(
        collector,
        container=listener_obj,
        required=required_keys,
        field_path=field_path,
    )
    if strict:
        _reject_unknown_keys(
            collector,
            container=listener_obj,
            allowed=required_keys,
            field_path=field_path,
        )

    mode = _validate_policy_mode(
        collector,
        listener_obj.get("mode"),
        f"{field_path}.mode",
    )
    allow_plaintext = _validate_bool(
        collector,
        listener_obj.get("allow_plaintext"),
        f"{field_path}.allow_plaintext",
    )

    if mode is None or allow_plaintext is None:
        return None

    return ListenerPolicy(mode=mode, allow_plaintext=allow_plaintext)


def _validate_domains(
    collector: _Collector,
    raw_value: Any,
    field_path: str,
    *,
    strict: bool,
) -> dict[str, DomainPolicy] | None:
    domains_obj = _expect_mapping(collector, raw_value, field_path)
    if domains_obj is None:
        return None

    seen_domains: dict[str, str] = {}
    validated: dict[str, DomainPolicy] = {}

    for raw_domain, raw_policy in domains_obj.items():
        domain_key_path = f"{field_path}.{raw_domain}"
        if not isinstance(raw_domain, str):
            collector.add(
                field_path=domain_key_path,
                message="domain keys must be strings",
                suggestion="Use a domain name string as the key (for example partner.example).",
            )
            continue

        normalized_domain = _normalize_domain(raw_domain)
        if normalized_domain is None:
            collector.add(
                field_path=domain_key_path,
                message=f"invalid domain key '{raw_domain}'",
                suggestion="Use a valid DNS hostname (letters, digits, dots, hyphens).",
            )
            continue

        prior = seen_domains.get(normalized_domain)
        if prior is not None:
            collector.add(
                field_path=domain_key_path,
                message=(
                    f"duplicate domain policy after normalization: '{raw_domain}' "
                    f"conflicts with '{prior}'"
                ),
                suggestion="Keep only one rule per normalized domain.",
            )
            continue

        seen_domains[normalized_domain] = raw_domain
        domain_policy = _validate_domain_policy(
            collector,
            raw_policy,
            domain_key_path,
            strict=strict,
        )
        if domain_policy is not None:
            validated[normalized_domain] = domain_policy

    return dict(sorted(validated.items(), key=lambda item: item[0]))


def _validate_domain_policy(
    collector: _Collector,
    raw_value: Any,
    field_path: str,
    *,
    strict: bool,
) -> DomainPolicy | None:
    domain_obj = _expect_mapping(collector, raw_value, field_path)
    if domain_obj is None:
        return None

    required_keys = {"mode"}
    allowed_keys = {"mode", "on_mismatch"}

    _require_keys(
        collector,
        container=domain_obj,
        required=required_keys,
        field_path=field_path,
    )
    if strict:
        _reject_unknown_keys(
            collector,
            container=domain_obj,
            allowed=allowed_keys,
            field_path=field_path,
        )

    mode = _validate_policy_mode(
        collector,
        domain_obj.get("mode"),
        f"{field_path}.mode",
    )
    mismatch_action = _validate_mismatch_action(
        collector,
        domain_obj.get("on_mismatch"),
        f"{field_path}.on_mismatch",
    )

    if mode is None:
        return None

    if mode == PolicyMode.REQUIRE_PQ:
        if mismatch_action is None:
            mismatch_action = MismatchAction.DEFER
        return DomainPolicy(mode=mode, on_mismatch=mismatch_action)

    if "on_mismatch" in domain_obj:
        collector.add(
            field_path=f"{field_path}.on_mismatch",
            message="on_mismatch is only valid when mode is require-pq",
            suggestion="Remove on_mismatch or set mode to require-pq.",
        )
        return None

    return DomainPolicy(mode=mode, on_mismatch=None)


def _validate_capability_hints(
    collector: _Collector,
    raw_value: Any,
    field_path: str,
    *,
    strict: bool,
) -> CapabilityHints | None:
    hints_obj = _expect_mapping(collector, raw_value, field_path)
    if hints_obj is None:
        return None

    allowed_keys = {"dns_txt"}
    if strict:
        _reject_unknown_keys(
            collector,
            container=hints_obj,
            allowed=allowed_keys,
            field_path=field_path,
        )

    dns_txt_raw = hints_obj.get("dns_txt")
    if dns_txt_raw is None:
        return CapabilityHints(dns_txt=None)

    dns_txt_obj = _expect_mapping(collector, dns_txt_raw, f"{field_path}.dns_txt")
    if dns_txt_obj is None:
        return None

    required_dns_keys = {"enabled"}
    allowed_dns_keys = {"enabled", "label"}

    _require_keys(
        collector,
        container=dns_txt_obj,
        required=required_dns_keys,
        field_path=f"{field_path}.dns_txt",
    )
    if strict:
        _reject_unknown_keys(
            collector,
            container=dns_txt_obj,
            allowed=allowed_dns_keys,
            field_path=f"{field_path}.dns_txt",
        )

    enabled = _validate_bool(
        collector,
        dns_txt_obj.get("enabled"),
        f"{field_path}.dns_txt.enabled",
    )
    label_raw = dns_txt_obj.get("label", "_verzola._tcp")
    label = _validate_dns_label(
        collector,
        label_raw,
        f"{field_path}.dns_txt.label",
    )

    if enabled is None or label is None:
        return None

    return CapabilityHints(dns_txt=DnsTxtHint(enabled=enabled, label=label))


def _validate_policy_mode(
    collector: _Collector,
    raw_value: Any,
    field_path: str,
) -> PolicyMode | None:
    if raw_value is None:
        collector.add(
            field_path=field_path,
            message="missing required field",
            suggestion="Set mode to one of: opportunistic, require-tls, require-pq.",
        )
        return None

    if not isinstance(raw_value, str):
        collector.add(
            field_path=field_path,
            message=f"expected string policy mode, got {type(raw_value).__name__}",
            suggestion="Use a quoted string mode value.",
        )
        return None

    try:
        return PolicyMode(raw_value)
    except ValueError:
        suggestion = _best_match(raw_value, [mode.value for mode in PolicyMode])
        collector.add(
            field_path=field_path,
            message=f"unknown policy mode '{raw_value}'",
            suggestion=(
                f"Use one of: opportunistic, require-tls, require-pq."
                + (f" Closest match: '{suggestion}'." if suggestion else "")
            ),
        )
        return None


def _validate_mismatch_action(
    collector: _Collector,
    raw_value: Any,
    field_path: str,
) -> MismatchAction | None:
    if raw_value is None:
        return None
    if not isinstance(raw_value, str):
        collector.add(
            field_path=field_path,
            message=f"expected string mismatch action, got {type(raw_value).__name__}",
            suggestion="Use 'defer' or 'reject'.",
        )
        return None

    try:
        return MismatchAction(raw_value)
    except ValueError:
        suggestion = _best_match(raw_value, [action.value for action in MismatchAction])
        collector.add(
            field_path=field_path,
            message=f"unknown mismatch action '{raw_value}'",
            suggestion=(
                "Use one of: defer, reject."
                + (f" Closest match: '{suggestion}'." if suggestion else "")
            ),
        )
        return None


def _validate_bool(
    collector: _Collector,
    raw_value: Any,
    field_path: str,
) -> bool | None:
    if raw_value is None:
        collector.add(
            field_path=field_path,
            message="missing required field",
            suggestion="Set the field to true or false.",
        )
        return None
    if not isinstance(raw_value, bool):
        collector.add(
            field_path=field_path,
            message=f"expected boolean, got {type(raw_value).__name__}",
            suggestion="Set the field to true or false.",
        )
        return None
    return raw_value


def _validate_dns_label(
    collector: _Collector,
    raw_value: Any,
    field_path: str,
) -> str | None:
    if not isinstance(raw_value, str):
        collector.add(
            field_path=field_path,
            message=f"expected string, got {type(raw_value).__name__}",
            suggestion="Set label to a DNS prefix like '_verzola._tcp'.",
        )
        return None

    label = raw_value.strip()
    if not label:
        collector.add(
            field_path=field_path,
            message="label must not be empty",
            suggestion="Use a non-empty value such as '_verzola._tcp'.",
        )
        return None

    if not label.startswith("_"):
        collector.add(
            field_path=field_path,
            message="label should start with '_' for service-style TXT hints",
            suggestion="Use '_verzola._tcp' or another underscore-prefixed label.",
        )
        return None

    return label


def _expect_mapping(
    collector: _Collector,
    raw_value: Any,
    field_path: str,
) -> dict[str, Any] | None:
    if raw_value is None:
        collector.add(
            field_path=field_path,
            message="missing required object",
            suggestion="Provide an object at this path.",
        )
        return None

    if not isinstance(raw_value, Mapping):
        collector.add(
            field_path=field_path,
            message=f"expected object/map, got {type(raw_value).__name__}",
            suggestion="Use key/value object syntax.",
        )
        return None

    return dict(raw_value)


def _require_keys(
    collector: _Collector,
    *,
    container: Mapping[str, Any],
    required: set[str],
    field_path: str,
) -> None:
    for key in sorted(required):
        if key not in container:
            path = _join_path(field_path, key)
            collector.add(
                field_path=path,
                message="missing required field",
                suggestion=f"Add '{key}' to this object.",
            )


def _reject_unknown_keys(
    collector: _Collector,
    *,
    container: Mapping[str, Any],
    allowed: set[str],
    field_path: str,
) -> None:
    for key in sorted(container.keys()):
        if key in allowed:
            continue
        suggestion = _best_match(str(key), sorted(allowed))
        collector.add(
            field_path=_join_path(field_path, str(key)),
            message=f"unknown field '{key}'",
            suggestion=(
                f"Use one of: {', '.join(sorted(allowed))}."
                + (f" Closest match: '{suggestion}'." if suggestion else "")
            ),
        )


def _normalize_domain(raw_domain: str) -> str | None:
    normalized = raw_domain.strip().strip(".").lower()
    if not normalized:
        return None
    if DOMAIN_PATTERN.fullmatch(normalized) is None:
        return None
    return normalized


def _best_match(raw_value: str, options: list[str]) -> str | None:
    matches = get_close_matches(raw_value, options, n=1, cutoff=0.6)
    return matches[0] if matches else None


def _join_path(base: str, child: str) -> str:
    if base in {"", "<root>"}:
        return child
    return f"{base}.{child}"


def _is_int(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool)

