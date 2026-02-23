from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum


class PolicyMode(str, Enum):
    OPPORTUNISTIC = "opportunistic"
    REQUIRE_TLS = "require-tls"
    REQUIRE_PQ = "require-pq"


class MismatchAction(str, Enum):
    DEFER = "defer"
    REJECT = "reject"


@dataclass(frozen=True)
class ListenerPolicy:
    mode: PolicyMode
    allow_plaintext: bool


@dataclass(frozen=True)
class ListenerSet:
    inbound: ListenerPolicy
    outbound: ListenerPolicy


@dataclass(frozen=True)
class DomainPolicy:
    mode: PolicyMode
    on_mismatch: MismatchAction | None = None


@dataclass(frozen=True)
class DnsTxtHint:
    enabled: bool
    label: str = "_verzola._tcp"


@dataclass(frozen=True)
class CapabilityHints:
    dns_txt: DnsTxtHint | None = None


@dataclass(frozen=True)
class PolicyConfig:
    version: int
    listeners: ListenerSet
    domains: dict[str, DomainPolicy] = field(default_factory=dict)
    capability_hints: CapabilityHints = field(default_factory=CapabilityHints)

