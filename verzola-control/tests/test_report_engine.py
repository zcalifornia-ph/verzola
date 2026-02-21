from __future__ import annotations

from contextlib import redirect_stderr, redirect_stdout
import io
import json
from pathlib import Path
import textwrap
import unittest

from verzola_control.cli import main
from verzola_control.report.engine import report_policy_file


POLICY_WITH_GAPS_YAML = """
version: 1
listeners:
  inbound:
    mode: opportunistic
    allow_plaintext: true
  outbound:
    mode: opportunistic
    allow_plaintext: false
domains:
  legacy.example:
    mode: opportunistic
  partner.example:
    mode: require-pq
    on_mismatch: defer
"""


POLICY_WITH_GAPS_TOML = """
version = 1

[listeners.inbound]
mode = "opportunistic"
allow_plaintext = true

[listeners.outbound]
mode = "opportunistic"
allow_plaintext = false

[domains."legacy.example"]
mode = "opportunistic"

[domains."partner.example"]
mode = "require-pq"
on_mismatch = "defer"
"""


POLICY_STRICT_YAML = """
version: 1
listeners:
  inbound:
    mode: require-tls
    allow_plaintext: false
  outbound:
    mode: require-tls
    allow_plaintext: false
domains:
  partner.example:
    mode: require-pq
    on_mismatch: reject
capability_hints:
  dns_txt:
    enabled: true
    label: _verzola._tcp
"""


class ReportEngineTests(unittest.TestCase):
    def setUp(self) -> None:
        self.base_path = Path.cwd() / ".tmp-tests" / "report" / self._testMethodName
        self.base_path.mkdir(parents=True, exist_ok=True)

    def _write(self, relative_path: str, content: str) -> Path:
        path = self.base_path / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(textwrap.dedent(content).strip() + "\n", encoding="utf-8")
        return path

    def test_report_json_includes_posture_and_gap_severity(self) -> None:
        policy_path = self._write("policy.yaml", POLICY_WITH_GAPS_YAML)

        output = report_policy_file(
            policy_path,
            environment="dev",
            output_format="json",
        )
        report = json.loads(output)

        self.assertEqual(report["environment"], "dev")
        self.assertEqual(report["posture_summary"]["domain_policy_count"], 2)
        self.assertEqual(report["posture_summary"]["require_pq_domain_count"], 1)
        severities = {gap["severity"] for gap in report["detected_gaps"]}
        self.assertIn("critical", severities)
        self.assertIn("warning", severities)
        self.assertIn("info", severities)
        self.assertTrue(any(gap["code"] == "U3R-001" for gap in report["detected_gaps"]))

    def test_report_json_is_deterministic_across_yaml_and_toml(self) -> None:
        yaml_path = self._write("policy.yaml", POLICY_WITH_GAPS_YAML)
        toml_path = self._write("policy.toml", POLICY_WITH_GAPS_TOML)

        report_yaml = json.loads(
            report_policy_file(
                yaml_path,
                environment="staging",
                output_format="json",
            )
        )
        report_toml = json.loads(
            report_policy_file(
                toml_path,
                environment="staging",
                output_format="json",
            )
        )

        report_yaml.pop("policy_file", None)
        report_toml.pop("policy_file", None)
        self.assertEqual(report_yaml, report_toml)


class CliReportTests(unittest.TestCase):
    def setUp(self) -> None:
        self.base_path = Path.cwd() / ".tmp-tests" / "cli-report" / self._testMethodName
        self.base_path.mkdir(parents=True, exist_ok=True)

    def _write(self, relative_path: str, content: str) -> Path:
        path = self.base_path / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(textwrap.dedent(content).strip() + "\n", encoding="utf-8")
        return path

    def test_cli_report_returns_zero_for_valid_policy(self) -> None:
        policy_path = self._write("ok.yaml", POLICY_STRICT_YAML)
        stdout = io.StringIO()
        stderr = io.StringIO()

        with redirect_stdout(stdout), redirect_stderr(stderr):
            exit_code = main(["report", str(policy_path), "--environment", "staging"])

        self.assertEqual(exit_code, 0)
        self.assertIn("POLICY REPORT", stdout.getvalue())
        self.assertIn("Environment: staging", stdout.getvalue())
        self.assertEqual(stderr.getvalue(), "")

    def test_cli_report_writes_json_file_for_sample_repo_layout(self) -> None:
        policy_path = self._write(
            "sample-repo/policies/policy.yaml",
            POLICY_WITH_GAPS_YAML,
        )
        output_path = self.base_path / "sample-repo" / "reports" / "policy-report.json"
        stdout = io.StringIO()
        stderr = io.StringIO()

        with redirect_stdout(stdout), redirect_stderr(stderr):
            exit_code = main(
                [
                    "report",
                    str(policy_path),
                    "--environment",
                    "prod",
                    "--format",
                    "json",
                    "--output",
                    str(output_path),
                ]
            )

        self.assertEqual(exit_code, 0)
        self.assertTrue(output_path.exists())
        report = json.loads(output_path.read_text(encoding="utf-8"))
        self.assertEqual(report["environment"], "prod")
        self.assertIn("detected_gaps", report)
        self.assertIn("report generated successfully", stdout.getvalue())
        self.assertEqual(stderr.getvalue(), "")

    def test_cli_report_returns_one_for_invalid_policy(self) -> None:
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
        stdout = io.StringIO()
        stderr = io.StringIO()

        with redirect_stdout(stdout), redirect_stderr(stderr):
            exit_code = main(["report", str(policy_path)])

        self.assertEqual(exit_code, 1)
        self.assertEqual(stdout.getvalue(), "")
        self.assertIn("unsupported schema version '7'", stderr.getvalue())


if __name__ == "__main__":
    unittest.main()

