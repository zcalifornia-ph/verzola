from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
import json
from pathlib import Path

from ..policy.model import MismatchAction, PolicyConfig, PolicyMode
from ..render.engine import resolve_environment_profile
from ..validate.engine import validate_policy_file

SUPPORTED_REPORT_FORMATS = ("json", "text")


class PolicyReportSeverity(str, Enum):
    CRITICAL = "critical"
    WARNING = "warning"
    INFO = "info"


@dataclass(frozen=True)
class PolicyGap:
    code: str
    severity: PolicyReportSeverity
    scope: str
    message: str
    recommendation: str

    def to_dict(self) -> dict[str, str]:
        return {
            "code": self.code,
            "message": self.message,
            "recommendation": self.recommendation,
            "scope": self.scope,
            "severity": self.severity.value,
        }


@dataclass(frozen=True)
class ReportDomainPolicy:
    domain: str
    mode: str
    on_mismatch: str | None

    def to_dict(self) -> dict[str, str]:
        rendered = {
            "domain": self.domain,
            "mode": self.mode,
        }
        if self.on_mismatch is not None:
            rendered["on_mismatch"] = self.on_mismatch
        return rendered


@dataclass(frozen=True)
class ReportPostureSummary:
    inbound_mode: str
    inbound_allow_plaintext: bool
    outbound_mode: str
    outbound_allow_plaintext: bool
    domain_policy_count: int
    domain_mode_counts: dict[str, int]
    require_pq_domain_count: int
    dns_txt_hints_enabled: bool
    dns_txt_label: str | None

    def to_dict(self) -> dict[str, object]:
        return {
            "dns_txt_hints_enabled": self.dns_txt_hints_enabled,
            "dns_txt_label": self.dns_txt_label,
            "domain_mode_counts": self.domain_mode_counts,
            "domain_policy_count": self.domain_policy_count,
            "inbound_allow_plaintext": self.inbound_allow_plaintext,
            "inbound_mode": self.inbound_mode,
            "outbound_allow_plaintext": self.outbound_allow_plaintext,
            "outbound_mode": self.outbound_mode,
            "require_pq_domain_count": self.require_pq_domain_count,
        }


@dataclass(frozen=True)
class PolicyReport:
    policy_file: str
    schema_version: int
    environment: str
    strict_mode: bool
    posture_summary: ReportPostureSummary
    domain_overrides: tuple[ReportDomainPolicy, ...]
    detected_gaps: tuple[PolicyGap, ...]

    def to_dict(self) -> dict[str, object]:
        return {
            "detected_gaps": [gap.to_dict() for gap in self.detected_gaps],
            "domain_overrides": [policy.to_dict() for policy in self.domain_overrides],
            "environment": self.environment,
            "policy_file": self.policy_file,
            "posture_summary": self.posture_summary.to_dict(),
            "schema_version": self.schema_version,
            "strict_mode": self.strict_mode,
        }


def build_policy_report(
    policy_config: PolicyConfig,
    *,
    policy_file: str | Path,
    environment: str,
    strict: bool,
) -> PolicyReport:
    profile = resolve_environment_profile(environment)
    policy_file_path = str(Path(policy_file))
    domain_overrides = tuple(
        ReportDomainPolicy(
            domain=domain,
            mode=domain_policy.mode.value,
            on_mismatch=(
                domain_policy.on_mismatch.value
                if domain_policy.on_mismatch is not None
                else None
            ),
        )
        for domain, domain_policy in sorted(policy_config.domains.items(), key=lambda item: item[0])
    )

    mode_counts: dict[str, int] = {
        PolicyMode.OPPORTUNISTIC.value: 0,
        PolicyMode.REQUIRE_TLS.value: 0,
        PolicyMode.REQUIRE_PQ.value: 0,
    }
    require_pq_domain_count = 0
    for domain_policy in domain_overrides:
        mode_counts[domain_policy.mode] += 1
        if domain_policy.mode == PolicyMode.REQUIRE_PQ.value:
            require_pq_domain_count += 1

    dns_txt = policy_config.capability_hints.dns_txt
    posture_summary = ReportPostureSummary(
        inbound_mode=policy_config.listeners.inbound.mode.value,
        inbound_allow_plaintext=policy_config.listeners.inbound.allow_plaintext,
        outbound_mode=policy_config.listeners.outbound.mode.value,
        outbound_allow_plaintext=policy_config.listeners.outbound.allow_plaintext,
        domain_policy_count=len(domain_overrides),
        domain_mode_counts=mode_counts,
        require_pq_domain_count=require_pq_domain_count,
        dns_txt_hints_enabled=dns_txt.enabled if dns_txt is not None else False,
        dns_txt_label=dns_txt.label if dns_txt is not None else None,
    )

    detected_gaps = tuple(_detect_gaps(policy_config, domain_overrides))
    return PolicyReport(
        policy_file=policy_file_path,
        schema_version=policy_config.version,
        environment=profile.name,
        strict_mode=strict,
        posture_summary=posture_summary,
        domain_overrides=domain_overrides,
        detected_gaps=detected_gaps,
    )


def report_policy_file(
    path: str | Path,
    *,
    environment: str,
    strict: bool = True,
    output_format: str = "text",
) -> str:
    validated = validate_policy_file(path, strict=strict)
    report = build_policy_report(
        validated,
        policy_file=path,
        environment=environment,
        strict=strict,
    )
    return render_policy_report(report, output_format=output_format)


def render_policy_report(
    report: PolicyReport,
    *,
    output_format: str,
) -> str:
    normalized = output_format.strip().lower()
    if normalized not in SUPPORTED_REPORT_FORMATS:
        supported = ", ".join(SUPPORTED_REPORT_FORMATS)
        raise ValueError(
            f"unsupported report format '{output_format}'; use one of: {supported}"
        )

    if normalized == "json":
        return json.dumps(report.to_dict(), indent=2, sort_keys=True)
    return _render_report_text(report)


def _detect_gaps(
    policy_config: PolicyConfig,
    domain_overrides: tuple[ReportDomainPolicy, ...],
) -> list[PolicyGap]:
    gaps: list[PolicyGap] = []

    if policy_config.listeners.inbound.allow_plaintext:
        gaps.append(
            PolicyGap(
                code="U3R-001",
                severity=PolicyReportSeverity.CRITICAL,
                scope="listeners.inbound.allow_plaintext",
                message="Inbound listener allows plaintext SMTP sessions.",
                recommendation="Set listeners.inbound.allow_plaintext to false.",
            )
        )

    if policy_config.listeners.outbound.allow_plaintext:
        gaps.append(
            PolicyGap(
                code="U3R-002",
                severity=PolicyReportSeverity.CRITICAL,
                scope="listeners.outbound.allow_plaintext",
                message="Outbound listener allows plaintext SMTP sessions.",
                recommendation="Set listeners.outbound.allow_plaintext to false.",
            )
        )

    if policy_config.listeners.outbound.mode == PolicyMode.OPPORTUNISTIC:
        gaps.append(
            PolicyGap(
                code="U3R-003",
                severity=PolicyReportSeverity.WARNING,
                scope="listeners.outbound.mode",
                message="Outbound default mode is opportunistic.",
                recommendation=(
                    "Use require-tls for stricter transport guarantees on default outbound traffic."
                ),
            )
        )

    require_pq_domains = [
        policy for policy in domain_overrides if policy.mode == PolicyMode.REQUIRE_PQ.value
    ]
    if not require_pq_domains:
        gaps.append(
            PolicyGap(
                code="U3R-004",
                severity=PolicyReportSeverity.WARNING,
                scope="domains",
                message="No domains are configured with require-pq policy.",
                recommendation=(
                    "Add partner domains with mode=require-pq where downgrade resistance is mandatory."
                ),
            )
        )

    for policy in domain_overrides:
        if policy.mode == PolicyMode.OPPORTUNISTIC.value:
            gaps.append(
                PolicyGap(
                    code="U3R-005",
                    severity=PolicyReportSeverity.WARNING,
                    scope=f"domains.{policy.domain}.mode",
                    message=f"Domain '{policy.domain}' uses opportunistic mode.",
                    recommendation=(
                        "Use require-tls or require-pq for domains that require stronger delivery controls."
                    ),
                )
            )

    for policy in require_pq_domains:
        if policy.on_mismatch == MismatchAction.DEFER.value:
            gaps.append(
                PolicyGap(
                    code="U3R-006",
                    severity=PolicyReportSeverity.INFO,
                    scope=f"domains.{policy.domain}.on_mismatch",
                    message=(
                        f"Domain '{policy.domain}' uses defer on PQ mismatch, which prioritizes retry safety."
                    ),
                    recommendation=(
                        "Consider reject when policy requires immediate hard failure for non-PQ peers."
                    ),
                )
            )

    dns_txt = policy_config.capability_hints.dns_txt
    if dns_txt is None or not dns_txt.enabled:
        gaps.append(
            PolicyGap(
                code="U3R-007",
                severity=PolicyReportSeverity.INFO,
                scope="capability_hints.dns_txt",
                message="DNS TXT capability hints are disabled.",
                recommendation=(
                    "Enable capability_hints.dns_txt for advisory partner capability signaling."
                ),
            )
        )

    return sorted(
        gaps,
        key=lambda gap: (
            _severity_rank(gap.severity),
            gap.code,
            gap.scope,
        ),
    )


def _severity_rank(severity: PolicyReportSeverity) -> int:
    if severity == PolicyReportSeverity.CRITICAL:
        return 0
    if severity == PolicyReportSeverity.WARNING:
        return 1
    return 2


def _render_report_text(report: PolicyReport) -> str:
    lines: list[str] = []
    lines.append("POLICY REPORT")
    lines.append(f"Policy File: {report.policy_file}")
    lines.append(f"Schema Version: {report.schema_version}")
    lines.append(f"Environment: {report.environment}")
    lines.append(f"Strict Mode: {'enabled' if report.strict_mode else 'disabled'}")
    lines.append("")

    summary = report.posture_summary
    lines.append("Posture Summary")
    lines.append(
        "  Inbound Default: "
        f"mode={summary.inbound_mode}, allow_plaintext={str(summary.inbound_allow_plaintext).lower()}"
    )
    lines.append(
        "  Outbound Default: "
        f"mode={summary.outbound_mode}, allow_plaintext={str(summary.outbound_allow_plaintext).lower()}"
    )
    lines.append(f"  Domain Overrides: {summary.domain_policy_count}")
    lines.append(
        "  Domain Modes: "
        f"opportunistic={summary.domain_mode_counts[PolicyMode.OPPORTUNISTIC.value]}, "
        f"require-tls={summary.domain_mode_counts[PolicyMode.REQUIRE_TLS.value]}, "
        f"require-pq={summary.domain_mode_counts[PolicyMode.REQUIRE_PQ.value]}"
    )
    lines.append(f"  Require-PQ Domains: {summary.require_pq_domain_count}")
    if summary.dns_txt_hints_enabled:
        lines.append(f"  DNS TXT Hints: enabled ({summary.dns_txt_label})")
    else:
        lines.append("  DNS TXT Hints: disabled")
    lines.append("")

    lines.append(f"Domain Overrides Detail ({len(report.domain_overrides)})")
    if not report.domain_overrides:
        lines.append("  - none")
    else:
        for policy in report.domain_overrides:
            mismatch = f", on_mismatch={policy.on_mismatch}" if policy.on_mismatch else ""
            lines.append(f"  - {policy.domain}: mode={policy.mode}{mismatch}")
    lines.append("")

    lines.append(f"Detected Gaps ({len(report.detected_gaps)})")
    if not report.detected_gaps:
        lines.append("  - none")
    else:
        for gap in report.detected_gaps:
            lines.append(
                "  - "
                f"[{gap.severity.value}] {gap.code} {gap.scope}: {gap.message} "
                f"Recommendation: {gap.recommendation}"
            )

    return "\n".join(lines)

