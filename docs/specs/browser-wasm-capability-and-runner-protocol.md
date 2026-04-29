# Spec: Browser/WASM Capability and Runner Protocol

Status: Draft
Date: 2026-04-29

## Purpose
Define capability boundaries and browser runner protocol semantics.

## Capability contract
- browser_safe/rootless_safe definitions
- allowed mode/payload matrix
- native-only exclusions
- capability-change approval rule

## Runner protocol contract
- RunMessage schema and protocolVersion
- request/response/error shape
- unsupported-mode behavior
- worker lifecycle and version payload semantics
