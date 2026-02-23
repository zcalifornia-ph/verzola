from __future__ import annotations

from pathlib import Path
import textwrap
import unittest

from verzola_control.cli import main
from verzola_control.policy.model import MismatchAction, PolicyMode
from verzola_control.validate.engine import PolicyValidationError, validate_policy_file


VALID_YAML = """
version: 1
listeners:
  inbound:
    mode: opportunistic
    allow_plaintext: false
  outbound:
    mode: opportunistic
    allow_plaintext: false
domains:
  partner.example:
    mode: require-pq
    on_mismatch: defer
  legacy.example:
    mode: require-tls
capability_hints:
  dns_txt:
    enabled: true
    label: _verzola._tcp
"""


VALID_TOML = """
version = 1

[listeners.inbound]
mode = "opportunistic"
allow_plaintext = false

[listeners.outbound]
mode = "require-tls"
allow_plaintext = false

[domains."partner.example"]
mode = "require-pq"
on_mismatch = "defer"

[capability_hints.dns_txt]
enabled = true
label = "_verzola._tcp"
"""


class ValidationEngineTests(unittest.TestCase):
    def setUp(self) -> None:
        self.base_path = (
            Path.cwd() / ".tmp-tests" / "validation" / self._testMethodName
        )
        self.base_path.mkdir(parents=True, exist_ok=True)

    def _write(self, relative_path: str, content: str) -> Path:
        path = self.base_path / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(textwrap.dedent(content).strip() + "\n", encoding="utf-8")
        return path

    def test_validate_yaml_sample_success(self) -> None:
        policy_path = self._write("policy.yaml", VALID_YAML)

        config = validate_policy_file(policy_path)

        self.assertEqual(config.version, 1)
        self.assertEqual(config.listeners.inbound.mode, PolicyMode.OPPORTUNISTIC)
        self.assertEqual(config.listeners.outbound.mode, PolicyMode.OPPORTUNISTIC)
        self.assertEqual(config.domains["partner.example"].mode, PolicyMode.REQUIRE_PQ)
        self.assertEqual(
            config.domains["partner.example"].on_mismatch,
            MismatchAction.DEFER,
        )
        self.assertTrue(config.capability_hints.dns_txt is not None)
        self.assertEqual(config.capability_hints.dns_txt.label, "_verzola._tcp")

    def test_validate_toml_sample_success(self) -> None:
        policy_path = self._write("policy.toml", VALID_TOML)

        config = validate_policy_file(policy_path)

        self.assertEqual(config.version, 1)
        self.assertEqual(config.listeners.outbound.mode, PolicyMode.REQUIRE_TLS)
        self.assertEqual(config.domains["partner.example"].mode, PolicyMode.REQUIRE_PQ)
        self.assertEqual(
            config.domains["partner.example"].on_mismatch,
            MismatchAction.DEFER,
        )

    def test_strict_mode_rejects_unknown_listener_field(self) -> None:
        policy_path = self._write(
            "unknown-field.yaml",
            """
            version: 1
            listeners:
              inbound:
                mode: opportunistic
                allow_plaintxt: false
              outbound:
                mode: opportunistic
                allow_plaintext: false
            """,
        )

        with self.assertRaises(PolicyValidationError) as context:
            validate_policy_file(policy_path, strict=True)

        rendered = str(context.exception)
        self.assertIn("listeners.inbound.allow_plaintxt", rendered)
        self.assertIn("unknown field 'allow_plaintxt'", rendered)
        self.assertIn("allow_plaintext", rendered)

    def test_invalid_mode_value_is_rejected(self) -> None:
        policy_path = self._write(
            "invalid-mode.yaml",
            """
            version: 1
            listeners:
              inbound:
                mode: opportunstic
                allow_plaintext: false
              outbound:
                mode: opportunistic
                allow_plaintext: false
            """,
        )

        with self.assertRaises(PolicyValidationError) as context:
            validate_policy_file(policy_path, strict=True)

        rendered = str(context.exception)
        self.assertIn("unknown policy mode 'opportunstic'", rendered)
        self.assertIn("opportunistic", rendered)

    def test_domain_duplicates_after_normalization_are_rejected(self) -> None:
        policy_path = self._write(
            "dup-domain.yaml",
            """
            version: 1
            listeners:
              inbound:
                mode: opportunistic
                allow_plaintext: false
              outbound:
                mode: opportunistic
                allow_plaintext: false
            domains:
              Example.COM:
                mode: require-tls
              example.com:
                mode: require-tls
            """,
        )

        with self.assertRaises(PolicyValidationError) as context:
            validate_policy_file(policy_path, strict=True)

        self.assertIn("duplicate domain policy after normalization", str(context.exception))

    def test_on_mismatch_requires_require_pq_mode(self) -> None:
        policy_path = self._write(
            "bad-mismatch.yaml",
            """
            version: 1
            listeners:
              inbound:
                mode: opportunistic
                allow_plaintext: false
              outbound:
                mode: opportunistic
                allow_plaintext: false
            domains:
              legacy.example:
                mode: require-tls
                on_mismatch: defer
            """,
        )

        with self.assertRaises(PolicyValidationError) as context:
            validate_policy_file(policy_path, strict=True)

        self.assertIn(
            "on_mismatch is only valid when mode is require-pq",
            str(context.exception),
        )

    def test_malformed_yaml_is_rejected_with_parse_location(self) -> None:
        policy_path = self._write(
            "malformed.yaml",
            """
            version: 1
            listeners
              inbound:
                mode: opportunistic
                allow_plaintext: false
              outbound:
                mode: opportunistic
                allow_plaintext: false
            """,
        )

        with self.assertRaises(PolicyValidationError) as context:
            validate_policy_file(policy_path, strict=True)

        rendered = str(context.exception)
        self.assertIn("<parse>:line:2", rendered)
        self.assertIn("invalid YAML syntax", rendered)


class CliValidateTests(unittest.TestCase):
    def setUp(self) -> None:
        self.base_path = Path.cwd() / ".tmp-tests" / "cli" / self._testMethodName
        self.base_path.mkdir(parents=True, exist_ok=True)

    def _write(self, relative_path: str, content: str) -> Path:
        path = self.base_path / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(textwrap.dedent(content).strip() + "\n", encoding="utf-8")
        return path

    def test_cli_validate_returns_zero_for_valid_policy(self) -> None:
        policy_path = self._write("ok.yaml", VALID_YAML)

        exit_code = main(["validate", str(policy_path)])

        self.assertEqual(exit_code, 0)

    def test_cli_validate_returns_one_for_invalid_policy(self) -> None:
        policy_path = self._write(
            "bad.yaml",
            """
            version: 7
            listeners:
              inbound:
                mode: opportunistic
                allow_plaintext: false
              outbound:
                mode: opportunistic
                allow_plaintext: false
            """,
        )

        exit_code = main(["validate", str(policy_path)])

        self.assertEqual(exit_code, 1)


if __name__ == "__main__":
    unittest.main()
