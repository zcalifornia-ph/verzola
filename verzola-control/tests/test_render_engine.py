from __future__ import annotations

from contextlib import redirect_stderr, redirect_stdout
import io
from pathlib import Path
import textwrap
import unittest

from verzola_control.cli import main
from verzola_control.render.engine import render_policy_file


POLICY_YAML = """
version: 1
listeners:
  inbound:
    mode: opportunistic
    allow_plaintext: false
  outbound:
    mode: require-tls
    allow_plaintext: false
domains:
  Zeta.example:
    mode: require-tls
  partner.example:
    mode: require-pq
    on_mismatch: reject
capability_hints:
  dns_txt:
    enabled: true
    label: _verzola._tcp
"""


POLICY_TOML = """
version = 1

[listeners.inbound]
mode = "opportunistic"
allow_plaintext = false

[listeners.outbound]
mode = "require-tls"
allow_plaintext = false

[domains."partner.example"]
mode = "require-pq"
on_mismatch = "reject"

[domains."zeta.example"]
mode = "require-tls"

[capability_hints.dns_txt]
enabled = true
label = "_verzola._tcp"
"""


EXPECTED_DEV_RENDER = """
{
  "capability_hints": {
    "dns_txt": {
      "enabled": true,
      "label": "_verzola._tcp"
    }
  },
  "environment": "dev",
  "proxy": {
    "inbound": {
      "advertise_starttls": true,
      "allow_plaintext": false,
      "banner_host": "localhost",
      "bind_addr": "127.0.0.1:2525",
      "max_line_len": 4096,
      "tls_policy": "opportunistic"
    },
    "outbound": {
      "banner_host": "localhost",
      "bind_addr": "127.0.0.1:10025",
      "max_line_len": 4096,
      "per_domain_tls_policies": [
        {
          "domain": "partner.example",
          "mode": "require-pq",
          "on_mismatch": "reject"
        },
        {
          "domain": "zeta.example",
          "mode": "require-tls"
        }
      ],
      "tls_policy": "require-tls"
    }
  },
  "schema_version": 1
}
"""


class RenderEngineTests(unittest.TestCase):
    def setUp(self) -> None:
        self.base_path = Path.cwd() / ".tmp-tests" / "render" / self._testMethodName
        self.base_path.mkdir(parents=True, exist_ok=True)

    def _write(self, relative_path: str, content: str) -> Path:
        path = self.base_path / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(textwrap.dedent(content).strip() + "\n", encoding="utf-8")
        return path

    def test_render_yaml_snapshot_for_dev_environment(self) -> None:
        policy_path = self._write("policy.yaml", POLICY_YAML)

        rendered = render_policy_file(policy_path, environment="dev")

        self.assertEqual(
            rendered,
            textwrap.dedent(EXPECTED_DEV_RENDER).strip(),
        )

    def test_render_output_is_deterministic_across_yaml_and_toml(self) -> None:
        yaml_path = self._write("policy.yaml", POLICY_YAML)
        toml_path = self._write("policy.toml", POLICY_TOML)

        rendered_yaml = render_policy_file(yaml_path, environment="staging")
        rendered_toml = render_policy_file(toml_path, environment="staging")

        self.assertEqual(rendered_yaml, rendered_toml)

    def test_render_uses_environment_specific_profile_values(self) -> None:
        policy_path = self._write("policy.yaml", POLICY_YAML)

        rendered_dev = render_policy_file(policy_path, environment="dev")
        rendered_prod = render_policy_file(policy_path, environment="prod")

        self.assertIn('"environment": "dev"', rendered_dev)
        self.assertIn('"bind_addr": "127.0.0.1:2525"', rendered_dev)
        self.assertIn('"banner_host": "localhost"', rendered_dev)

        self.assertIn('"environment": "prod"', rendered_prod)
        self.assertIn('"bind_addr": "0.0.0.0:25"', rendered_prod)
        self.assertIn('"banner_host": "mail.verzola.local"', rendered_prod)


class CliRenderTests(unittest.TestCase):
    def setUp(self) -> None:
        self.base_path = Path.cwd() / ".tmp-tests" / "cli-render" / self._testMethodName
        self.base_path.mkdir(parents=True, exist_ok=True)

    def _write(self, relative_path: str, content: str) -> Path:
        path = self.base_path / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(textwrap.dedent(content).strip() + "\n", encoding="utf-8")
        return path

    def test_cli_render_returns_zero_for_valid_policy(self) -> None:
        policy_path = self._write("ok.yaml", POLICY_YAML)
        stdout = io.StringIO()
        stderr = io.StringIO()

        with redirect_stdout(stdout), redirect_stderr(stderr):
            exit_code = main(["render", str(policy_path), "--environment", "dev"])

        self.assertEqual(exit_code, 0)
        self.assertIn('"environment": "dev"', stdout.getvalue())
        self.assertEqual(stderr.getvalue(), "")

    def test_cli_render_writes_output_file_when_requested(self) -> None:
        policy_path = self._write("ok.yaml", POLICY_YAML)
        output_path = self.base_path / "rendered" / "effective-config.json"
        stdout = io.StringIO()
        stderr = io.StringIO()

        with redirect_stdout(stdout), redirect_stderr(stderr):
            exit_code = main(
                [
                    "render",
                    str(policy_path),
                    "--environment",
                    "staging",
                    "--output",
                    str(output_path),
                ]
            )

        self.assertEqual(exit_code, 0)
        self.assertTrue(output_path.exists())
        content = output_path.read_text(encoding="utf-8")
        self.assertIn('"environment": "staging"', content)
        self.assertIn("rendered successfully for environment 'staging'", stdout.getvalue())
        self.assertEqual(stderr.getvalue(), "")

    def test_cli_render_returns_one_for_invalid_policy(self) -> None:
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
            exit_code = main(["render", str(policy_path)])

        self.assertEqual(exit_code, 1)
        self.assertEqual(stdout.getvalue(), "")
        self.assertIn("unsupported schema version '7'", stderr.getvalue())


if __name__ == "__main__":
    unittest.main()
