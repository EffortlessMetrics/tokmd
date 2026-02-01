# Security Policy

## Supported Versions

| Version | Supported |
| ------- | --------- |
| 1.x     | Yes       |
| < 1.0   | No        |

## Reporting a Vulnerability

Please do not report security vulnerabilities through public GitHub issues.

### Preferred Methods

1. **GitHub Security Advisories** (recommended): Use the [Security Advisories](https://github.com/EffortlessMetrics/tokmd/security/advisories/new) feature to report vulnerabilities privately.

2. **Contact Form**: Send details to **https://effortlesssteven.com/about/**

### What to Include

- Description of the vulnerability
- Steps to reproduce or proof-of-concept
- Potential impact assessment
- Any suggested fixes (optional)

## Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 7 days
- **Resolution**: Within 30 days for critical issues, 90 days for lower severity

## What Constitutes a Security Issue

The following are considered security vulnerabilities for tokmd:

- **Path traversal**: Operations that access files outside intended directories
- **Arbitrary code execution**: Malicious input causing unintended code execution
- **Denial of service**: Input causing crashes, infinite loops, or excessive resource consumption
- **Information disclosure**: Unintended exposure of sensitive file contents or system information
- **Command injection**: Unsanitized input passed to shell commands or external processes

### Out of Scope

- Issues in unsupported versions
- Theoretical attacks without a realistic exploitation scenario
- Performance issues that do not constitute denial of service
- Bugs in third-party dependencies (report these upstream, but feel free to notify us)

## Disclosure Process

1. We will acknowledge receipt of your report promptly.
2. We will investigate and determine the severity and scope.
3. We will develop and test a fix.
4. We will coordinate disclosure timing with you.
5. We will credit you in the release notes (unless you prefer anonymity).

Thank you for helping keep tokmd secure.
