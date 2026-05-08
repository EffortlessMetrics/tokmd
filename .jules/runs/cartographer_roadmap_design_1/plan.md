1. **Update `docs/implementation-plan.md` to reflect `v1.11.0` is complete.**
   - Change `## Phase 5c: Browser Runtime Polish (v1.11.0)` to `## Phase 5c: Browser Runtime Polish (v1.11.0) ✅ Complete`

2. **Add Cockpit & Architecture Consolidation phases to `docs/implementation-plan.md`.**
   - Insert new Phase between 5c and 6, marking it as the active lane:
     ```markdown
     ## Phase 5d: Cockpit Hardening & Architecture Consolidation (Active)

     **Goal**: Improve cockpit as the PR-review evidence surface and collapse implementation microcrates into SRP modules.

     ### Work Items

     - [ ] Finish small cockpit review-packet and Action-hosting gaps
     - [ ] Preserve `tokmd cockpit` as the review evidence implementation surface
     - [ ] Consolidate architecture in batches, preserving `ci/proof.toml` scope granularity
     ```

3. **Update `ROADMAP.md` to reflect the active lanes in Future Horizons.**
   - Add a `v1.12.0` or active horizon phase for "Cockpit Review-Packet Hardening & Architecture Consolidation" right before `v2.0 — Platform Evolution`.
     ```markdown
     ### v1.12.0 — Cockpit & Architecture Consolidation (Active)

     _Goal: Improve cockpit as the PR-review evidence surface before adding a separate review orchestrator, and consolidate implementation microcrates into SRP modules._

     - Finish small cockpit review-packet and Action-hosting gaps.
     - Preserve `tokmd cockpit` as the review evidence implementation surface.
     - Start architecture consolidation in batches, preserving scope granularity.
     ```

4. **Verify the changes using `grep` and `cat`.**

5. **Generate required artifacts:**
   - Create `.jules/runs/cartographer_roadmap_design_1/result.json`.
   - Create `.jules/runs/cartographer_roadmap_design_1/pr_body.md`.

6. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
