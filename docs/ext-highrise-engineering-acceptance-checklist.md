# EXT High-Rise Engineering Acceptance Checklist

Reviewer: ____________________  
Date: ____________________  
Project: ____________________  
Version/Branch: ____________________  
TT Spreadsheet File + Revision: ____________________

## 1) Purpose
Use this checklist to verify `ext-calc` + `ext-render` outputs are acceptable for high-rise structural stress-test review, and reconcile with Thornton Tomasetti spreadsheet checks.

## 2) Required Review Package
- `calc_output.json` from target version
- Rendered chart assets (`images/*.svg` or HTML render output)
- Generated report PDF (if used for distribution)
- ETABS source model + controlling load combinations list
- TT spreadsheet with same model/version assumptions

## 3) Acceptance Checklist

| # | Item | What to Verify | Data Source | Acceptance Criteria | Status | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Units source consistency | Confirm calc/report units match engineering basis | config (`project.units`, `extract.units`), `calc_output.meta.units` | Declared project review units and output units are consistent; no silent unit drift | [ ] | |
| 2 | Drift tracking group mapping | Confirm configured tracking groups exist and include intended joints | config + `group_assignments.parquet` + drift rows | Every configured group resolves correctly and contains expected control joints | [ ] | |
| 3 | Torsional geometry inputs | Confirm `building-dim-x-ft` / `building-dim-y-ft` and `ecc-ratio` reflect project geometry | config + torsional output | Eccentricity basis matches design assumptions; no zero-dimension fallback | [ ] | |
| 4 | Modal participation threshold | Compare mode reach and cumulative mass to TT sheet | `modal` block + TT modal tab | Threshold mode and cumulative ratios match within engineering rounding tolerance | [ ] | |
| 5 | Base reaction RSA/ELF scaling | Compare directional ratios and pass/fail with spreadsheet | `baseReactions.directionX/Y` + TT base shear tab | X and Y ratio checks match expected values and criteria (`rsa-scale-min`) | [ ] | |
| 6 | Story force envelopes | Validate per-story VX/VY/MX/MY envelopes | `storyForces.rows` + TT story force tab | Governing stories and envelope magnitudes align within agreed tolerance | [ ] | |
| 7 | Wind drift envelopes | Validate governing story/case and DCR in X/Y | `driftWind.x/y` + TT drift tab | Demand, limit, and DCR trends and governing floors are consistent | [ ] | |
| 8 | Seismic drift envelopes | Validate governing story/case and DCR in X/Y | `driftSeismic.x/y` + TT drift tab | Demand, limit, and DCR trends and governing floors are consistent | [ ] | |
| 9 | Wind displacement limits | Validate `H / limit_h` implementation and governing displacements | `displacementWind.x/y` + story defs + TT displacement tab | Limit basis uses total building height and DCR matches spreadsheet logic | [ ] | |
| 10 | Pier shear stress logic | Validate per-pier stress equation inputs and governing rows | `pierShearStressWind/Seismic` + TT wall shear tab | Governing stress results match spreadsheet calculations; chart interpretation is unit-consistent | [ ] | |
| 11 | Pier axial stress logic | Validate `phi * 0.85 f'c Ag` basis and governing combos per category | `pierAxialStress` + TT wall axial tab | Governing gravity/wind/seismic results and DCRs match intended conservative method | [ ] | |
| 12 | Missing-data handling | Confirm behavior when combos/groups/cases are absent | logs + output presence/absence | Missing critical inputs are flagged per review policy (not silently accepted) | [ ] | |
| 13 | Chart-to-table consistency | Confirm chart series match table values (same governing trends) | chart assets + report JSON/table blocks | No contradictory values between plots and tables for same check | [ ] | |
| 14 | Report readiness | Confirm final report reflects approved engineering narrative | PDF + checklist | Only accepted checks are presented as decision-ready | [ ] | |

## 4) Numeric Tolerance Policy (fill before sign-off)
- Modal cumulative ratio tolerance: ____________________
- Base shear ratio tolerance: ____________________
- Story force tolerance (absolute or %): ____________________
- Drift/displacement tolerance (absolute or %): ____________________
- Pier stress/axial tolerance (absolute or %): ____________________

## 5) Governing Spot-Check Set (recommended minimum)
Record at least one manual check for each:
- Modal governing mode (UX and UY)
- Base reaction ratio X and Y
- Governing wind drift floor (X and Y)
- Governing seismic drift floor (X and Y)
- Governing wind displacement floor (X and Y)
- Worst pier shear row (wind and seismic)
- Worst pier axial row (gravity, wind, seismic)

## 6) Sign-Off
- Engineering Reviewer: ____________________  Date: ____________________
- Checker/Peer Reviewer: ____________________  Date: ____________________
- Decision: `Accepted` / `Accepted with comments` / `Revise and rerun`

