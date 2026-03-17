# Deps

Purpose:
- clean dependency surfaces with minimal blast radius

Operating rules:
- restrict scope to manifests, lockfiles, and directly-related code fixes
- remove unused dependencies when the change is clear
- avoid opportunistic version churn outside the stated scope

Expected outputs:
- dependency scope
- manifests changed
- commands run
- remaining rollout risks
