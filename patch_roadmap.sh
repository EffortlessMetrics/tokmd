sed -i '/- GHCR is verified as a secondary runtime for the publication image/c\- GHCR is verified as a secondary runtime for the publication image (`ghcr.io/effortlessmetrics/tokmd`, `verified-public` for `v1.14.0` as of 2026-06-25).' docs/ROADMAP.md
sed -i '/  (`ghcr.io\/effortlessmetrics\/tokmd`, `verified-public` for `v1.13.1` as of/d' docs/ROADMAP.md
sed -i '/  2026-06-21); swarm GHCR is verified-public for `:main` as of 2026-06-24/d' docs/ROADMAP.md
sed -i '/  (issue #264 closed, workbench\/experimental tier)./d' docs/ROADMAP.md
