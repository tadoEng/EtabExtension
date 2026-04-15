# EXT Council Readout (PM + Associate Review)

Project: ____________________  
Model / Version: ____________________  
Date: ____________________  
Presenter: ____________________

## 1) Decision Summary (RAG)

| Area | Status | Evidence | Gate |
| --- | --- | --- | --- |
| Input/Config Freeze | ⬜ Red ⬜ Amber ⬜ Green | Config snapshot + model hash/version | Same inputs reproduce same outputs |
| Calculation Traceability | ⬜ Red ⬜ Amber ⬜ Green | Formula + JSON path mapping | 100% of reported values traceable |
| Spreadsheet Parity (TT) | ⬜ Red ⬜ Amber ⬜ Green | Delta table vs TT benchmarks | Within approved tolerances |
| Governing Case Alignment | ⬜ Red ⬜ Amber ⬜ Green | Governing story/case comparison | 100% match for critical checks |
| Unit Consistency | ⬜ Red ⬜ Amber ⬜ Green | Unit audit (`config` vs output) | No unresolved unit-basis conflicts |
| Reporting Readiness | ⬜ Red ⬜ Amber ⬜ Green | Table/chart consistency + narrative | No black-box fields in final report |

Recommended decision: ⬜ Approve for use ⬜ Approve with actions ⬜ Hold pending fixes

## 2) What Was Verified (Measurable)

| Metric | Target | Actual | Pass/Fail |
| --- | --- | --- | --- |
| Traceability coverage (value -> formula -> source field) | 100% | ______ | ⬜ Pass ⬜ Fail |
| TT benchmark checks completed | >= 8 governing checks | ______ | ⬜ Pass ⬜ Fail |
| Benchmark parity within tolerance | >= 95% | ______ | ⬜ Pass ⬜ Fail |
| Governing-point parity | 100% | ______ | ⬜ Pass ⬜ Fail |
| Silent-pass missing-data cases | 0 critical | ______ | ⬜ Pass ⬜ Fail |
| Unit mismatch findings unresolved | 0 | ______ | ⬜ Pass ⬜ Fail |

## 3) Non-Black-Box Proof Pack

Show these artifacts live in the review:

1. Config freeze (`config.toml` + `config.local.toml` if relevant).
2. `calc_output.json` contract mapping:
   - config keys -> output JSON fields -> chart/table fields.
3. Formula sheet per check:
   - modal, base reactions, story forces, drift, displacement, pier shear, pier axial, torsional.
4. TT parity sheet:
   - expected vs EXT result, absolute delta, percent delta, tolerance, disposition.
5. Governing case sheet:
   - EXT governing story/case vs TT governing story/case.
6. Exception register:
   - known limitation, impact, mitigation, owner, due date.

## 4) Critical Risks to Call Out Explicitly

| Risk | Impact | Current Control | Owner | Due |
| --- | --- | --- | --- | --- |
| Unit source mismatch (`extract.units` vs calc unit context) | Misinterpreted demand/limit values | Unit audit before sign-off | ______ | ______ |
| Pier shear chart limit/demand basis interpretation | Chart may be misread for design decision | Use table values as authority until chart basis is aligned | ______ | ______ |
| Missing-data pass behavior in some paths | False confidence if inputs absent | Add explicit missing-input gate in review checklist | ______ | ______ |
| Drift/displacement chart aggregation hides case trace | Reduced transparency in chart-only view | Keep governing case table beside chart | ______ | ______ |

## 5) Action Log (if not full-green)

| Action | Severity | Owner | Due Date | Verification Method |
| --- | --- | --- | --- | --- |
| ____________________ | High / Med / Low | ______ | ______ | ______ |
| ____________________ | High / Med / Low | ______ | ______ | ______ |
| ____________________ | High / Med / Low | ______ | ______ | ______ |

## 6) Final Council Decision Record

- Decision: ⬜ Approved ⬜ Approved with conditions ⬜ Not approved  
- Conditions / required actions: ____________________________________________  
- Re-review required by: ____________________  
- Signatures:
  - PM: ____________________  Date: ____________________
  - Associate: ____________________  Date: ____________________
  - Structural Reviewer: ____________________  Date: ____________________

