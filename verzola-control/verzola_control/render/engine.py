from __future__ import annotations

from dataclasses import dataclass
import json
from pathlib import Path

from ..policy.model import PolicyConfig
from ..validate.engine import validate_policy_file


@dataclass(frozen=True)
class EnvironmentProfile:
    name: str
    inbound_bind_addr: str
    outbound_bind_addr: str
    banner_host: str
    max_line_len: int = 4096


@dataclass(frozen=True)
class RenderedDomainPolicy:
    domain: str
    mode: str
    on_mismatch: str | None = None

    def to_dict(self) -> dict[str, str]:
        rendered = {
            "domain": self.domain,
            "mode": self.mode,
        }
        if self.on_mismatch is not None:
            rendered["on_mismatch"] = self.on_mismatch
        return rendered


@dataclass(frozen=True)
class RenderedListenerConfig:
    bind_addr: str
    banner_host: str
    tls_policy: str
    allow_plaintext: bool
    advertise_starttls: bool
    max_line_len: int

    def to_dict(self) -> dict[str, object]:
        return {
            "advertise_starttls": self.advertise_starttls,
            "allow_plaintext": self.allow_plaintext,
            "banner_host": self.banner_host,
            "bind_addr": self.bind_addr,
            "max_line_len": self.max_line_len,
            "tls_policy": self.tls_policy,
        }


@dataclass(frozen=True)
class RenderedOutboundConfig:
    bind_addr: str
    banner_host: str
    tls_policy: str
    per_domain_tls_policies: tuple[RenderedDomainPolicy, ...]
    max_line_len: int

    def to_dict(self) -> dict[str, object]:
        return {
            "banner_host": self.banner_host,
            "bind_addr": self.bind_addr,
            "max_line_len": self.max_line_len,
            "per_domain_tls_policies": [
                rule.to_dict() for rule in self.per_domain_tls_policies
            ],
            "tls_policy": self.tls_policy,
        }


@dataclass(frozen=True)
class RenderedPolicyArtifact:
    schema_version: int
    environment: str
    capability_hints: dict[str, object]
    inbound: RenderedListenerConfig
    outbound: RenderedOutboundConfig

    def to_dict(self) -> dict[str, object]:
        return {
            "capability_hints": self.capability_hints,
            "environment": self.environment,
            "proxy": {
                "inbound": self.inbound.to_dict(),
                "outbound": self.outbound.to_dict(),
            },
            "schema_version": self.schema_version,
        }


_ENVIRONMENT_PROFILES: dict[str, EnvironmentProfile] = {
    "dev": EnvironmentProfile(
        name="dev",
        inbound_bind_addr="127.0.0.1:2525",
        outbound_bind_addr="127.0.0.1:10025",
        banner_host="localhost",
    ),
    "staging": EnvironmentProfile(
        name="staging",
        inbound_bind_addr="0.0.0.0:2525",
        outbound_bind_addr="0.0.0.0:10025",
        banner_host="staging.verzola.local",
    ),
    "prod": EnvironmentProfile(
        name="prod",
        inbound_bind_addr="0.0.0.0:25",
        outbound_bind_addr="127.0.0.1:10025",
        banner_host="mail.verzola.local",
    ),
}

SUPPORTED_ENVIRONMENTS = tuple(sorted(_ENVIRONMENT_PROFILES.keys()))


def resolve_environment_profile(environment: str) -> EnvironmentProfile:
    normalized = environment.strip().lower()
    profile = _ENVIRONMENT_PROFILES.get(normalized)
    if profile is None:
        supported = ", ".join(SUPPORTED_ENVIRONMENTS)
        raise ValueError(
            f"unsupported render environment '{environment}'; use one of: {supported}"
        )
    return profile


def build_rendered_config(
    policy_config: PolicyConfig,
    *,
    environment: str,
) -> RenderedPolicyArtifact:
    profile = resolve_environment_profile(environment)

    dns_txt = policy_config.capability_hints.dns_txt
    capability_hints = {
        "dns_txt": (
            {
                "enabled": dns_txt.enabled,
                "label": dns_txt.label,
            }
            if dns_txt is not None
            else None
        )
    }

    outbound_rules = tuple(
        RenderedDomainPolicy(
            domain=domain,
            mode=policy.mode.value,
            on_mismatch=policy.on_mismatch.value if policy.on_mismatch else None,
        )
        for domain, policy in sorted(policy_config.domains.items(), key=lambda item: item[0])
    )

    return RenderedPolicyArtifact(
        schema_version=policy_config.version,
        environment=profile.name,
        capability_hints=capability_hints,
        inbound=RenderedListenerConfig(
            bind_addr=profile.inbound_bind_addr,
            banner_host=profile.banner_host,
            tls_policy=policy_config.listeners.inbound.mode.value,
            allow_plaintext=policy_config.listeners.inbound.allow_plaintext,
            advertise_starttls=True,
            max_line_len=profile.max_line_len,
        ),
        outbound=RenderedOutboundConfig(
            bind_addr=profile.outbound_bind_addr,
            banner_host=profile.banner_host,
            tls_policy=policy_config.listeners.outbound.mode.value,
            per_domain_tls_policies=outbound_rules,
            max_line_len=profile.max_line_len,
        ),
    )


def render_policy_config(
    policy_config: PolicyConfig,
    *,
    environment: str,
) -> str:
    artifact = build_rendered_config(
        policy_config,
        environment=environment,
    )
    return json.dumps(artifact.to_dict(), indent=2, sort_keys=True)


def render_policy_file(
    path: str | Path,
    *,
    environment: str,
    strict: bool = True,
) -> str:
    validated = validate_policy_file(path, strict=strict)
    return render_policy_config(validated, environment=environment)
