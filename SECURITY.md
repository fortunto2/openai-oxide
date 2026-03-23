# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.9.x   | :white_check_mark: |
| < 0.9   | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly:

1. **Do NOT open a public GitHub issue**
2. Email: info@life2film.com
3. Include: description, reproduction steps, and impact assessment

We will acknowledge receipt within 48 hours and provide a fix timeline within 7 days.

## Security Practices

- No secrets in source code (pre-commit hook scans for API keys)
- Native bindings built in GitHub Actions CI with provenance
- No `child_process`, `eval`, or dynamic code execution in the npm package
- API keys read only from environment variables or explicit constructor parameters
- All dependencies pinned in lockfiles
