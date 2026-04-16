pub const INCLUDE_CALC_PROCEDURE_PAGE: bool = true;

pub fn append_definitions(doc: &mut String) {
    if !INCLUDE_CALC_PROCEDURE_PAGE {
        return;
    }

    doc.push_str(
        r#"
#let calc-procedure-page() = {
  text(size: parse-pt(theme.title-size), weight: "bold")[Calculation Procedure Notes]
  v(parse-pt(theme.section-gap))

  text(weight: "bold")[Torsional Irregularity (ext-calc)]
  v(4pt)
  [
    1. Read per-story/per-case diaphragm drift responses at two control joints (A and B).\
    2. Compute drift demand ratio as `delta-max / delta-avg`, where `delta-max = max(|A|, |B|)` and `delta-avg = (|A| + |B|)/2`.\
    3. Classify irregularity: Type A when ratio >= 1.2, Type B when ratio >= 1.4.\
    4. Store governing story/case, maximum ratio, and directional flags for report tables.
  ]

  v(12pt)
  text(weight: "bold")[Pier Shear Stress (ext-calc)]
  v(4pt)
  [
    1. Collect per-pier stress result rows (`story`, `pier`, `stress-psi`) and preserve top-to-bottom story order.\
    2. Evaluate stress-ratio limits with individual limit `10 * sqrt(f'c)` and average limit `8 * sqrt(f'c)`.\
    3. Group data into matrix tables with levels as rows and pier labels as columns (stress values in psi).\
    4. Render line charts by pier series with normalized pier ordering (`PX*`, then `PY*`, then others).
  ]
}
"#,
    );
}

pub fn append_sequence(doc: &mut String) {
    if !INCLUDE_CALC_PROCEDURE_PAGE {
        return;
    }
    doc.push_str("#pagebreak()\n#calc-procedure-page()\n\n");
}
