# Implementation task graph

**Generated from `plan/tasks.json`; this is not a second progress tracker.**

## T00 — Pin, inventory and reproduce the baseline

Status: **done** · Prerequisites: none

Pin source/toolchain; generate actual feature/dependency/capability evidence and isolated baseline. No broad code deletion.

Requirements: SP-001, SP-004, SP-024, SP-025, SP-026

Acceptance: A01, A45, A47

Gates to investigate: G-CHAT, G-PORT, G-FILE, G-OBS, G-PLATFORM

Gates this task must close: none; record findings without claiming full parity

## T01 — Isolate Spaceport development identity

Status: **in_progress** · Prerequisites: T00

Separate names/state/control endpoint from upstream; preserve compatibility and attribution. Production values remain gated.

Requirements: SP-001, SP-022, SP-025

Acceptance: A42, A47

Gates to investigate: G-RELEASE

Gates this task must close: none; record findings without claiming full parity

## T02 — Adapt the verified Core capability boundary

Status: **in_progress** · Prerequisites: T00

Typed Core calls/streams/capabilities with no legacy fallback or parallel remote manager.

Requirements: SP-003, SP-004, SP-005, SP-006, SP-019, SP-020

Acceptance: A02, A03, A04, A05, A53

Gates to investigate: none additional

Gates this task must close: none; record findings without claiming full parity

## T03 — Build one authoritative view projection

Status: **not_started** · Prerequisites: T02

Migrate readers from legacy snapshots; implement namespaced versioned presentation documents and local view preferences.

Requirements: SP-002, SP-007, SP-013, SP-018, SP-024

Acceptance: A06, A07, A08, A09, A24, A53

Gates to investigate: none additional

Gates this task must close: none; record findings without claiming full parity

## T04 — Remove excluded product feature families

Status: **not_started** · Prerequisites: T01, T03

Delete IDE/Git/GitHub/browser/cloud/Teleport/orchestration/mod/scheduler surfaces and dependencies in coherent units.

Requirements: SP-002, SP-015, SP-016, SP-022

Acceptance: A28, A29, A30, A33, A34

Gates to investigate: none additional

Gates this task must close: none; record findings without claiming full parity

## T05 — Prove terminal and lifecycle parity

Status: **not_started** · Prerequisites: T03

Local/remote Core terminal attach, input and window lifecycle through retained renderer; process survival proved.

Requirements: SP-002, SP-008, SP-012, SP-013, SP-018, SP-020

Acceptance: A09, A10, A11, A12, A13, A22, A23, A24, A40

Gates to investigate: none additional

Gates this task must close: none; record findings without claiming full parity

## T06 — Preserve native chat under the ownership contract

Status: **blocked** · Prerequisites: T03, T05

Keep renderer/host adapter where valid; remove desktop-owned conversion/resume/config; prove route or record blocked parity.

Requirements: SP-003, SP-009, SP-010, SP-019, SP-026

Acceptance: A14, A15, A16, A17, A54, A55

Gates to investigate: G-CHAT

Gates this task must close: G-CHAT

## T07 — Adapt hosts, transfers and service access

Status: **blocked** · Prerequisites: T02, T03

Preserve host/file/service UX through verified Core routes; no public rebinding or parallel transport workaround.

Requirements: SP-006, SP-014, SP-019, SP-026

Acceptance: A04, A05, A25, A26, A27, A28

Gates to investigate: G-PORT, G-FILE

Gates this task must close: G-PORT, G-FILE

## T08 — Preserve activity and attention without collectors

Status: **not_started** · Prerequisites: T03, T06

Existing timeline and notifications consume authoritative data; usage/job gaps explicit; no background service.

Requirements: SP-002, SP-010, SP-011, SP-012, SP-013, SP-021

Acceptance: A18, A19, A20, A21, A24, A41

Gates to investigate: G-OBS

Gates this task must close: G-OBS

## T09 — Unify client commands and structured inspection

Status: **not_started** · Prerequisites: T03, T05

One catalog for existing GUI/CLI; exact targets, bounded observations, honest outcomes and preserved confirmations.

Requirements: SP-017, SP-018, SP-019, SP-020, SP-021, SP-024

Acceptance: A16, A17, A35, A36, A37, A38, A39, A41, A45, A46, A52, A55

Gates to investigate: none additional

Gates this task must close: none; record findings without claiming full parity

## T10 — Delete legacy/mobile remnants and finalize dependencies

Status: **not_started** · Prerequisites: T04, T05, T06, T07, T08, T09

Remove all excluded reachable behavior and packaged services after necessary pure primitives are extracted.

Requirements: SP-004, SP-015, SP-016, SP-017, SP-022, SP-026

Acceptance: A29, A30, A31, A32, A42, A54

Gates to investigate: none additional

Gates this task must close: none; record findings without claiming full parity

## T11 — Run native desktop and end-to-end qualification

Status: **not_started** · Prerequisites: T10

Native platform, adversarial, resource and agent-usability evidence. Unproved requirements remain blocked.

Requirements: SP-001, SP-002, SP-008, SP-012, SP-018, SP-019, SP-020, SP-024, SP-026

Acceptance: A33, A34, A38, A39, A40, A46, A48, A49, A50, A51, A52

Gates to investigate: G-CHAT, G-PORT, G-FILE, G-OBS, G-PLATFORM

Gates this task must close: G-CHAT, G-PORT, G-FILE, G-OBS, G-PLATFORM

## T12 — Complete release identity and verified updates

Status: **blocked** · Prerequisites: T01, T10, T11

Owner-controlled signed packages and updates; no publishing without authorization; all product gates reconciled.

Requirements: SP-022, SP-023, SP-025, SP-026

Acceptance: A42, A43, A44, A48, A49, A50, A51

Gates to investigate: G-RELEASE, G-PLATFORM

Gates this task must close: G-RELEASE, G-PLATFORM
