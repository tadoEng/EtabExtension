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
    1. Collect per-pier stress result rows (`story`, `pier`, `combo`, `stress-psi`, `limit-individual`, `stress-ratio`).\
    2. Compute demand-capacity ratio as `dcr = stress-ratio / limit-individual`.\
    3. Sort rows by descending `dcr` to expose governing combinations first.\
    4. Apply report annotations: `fail` for dcr >= 1.0, `warn` for dcr >= 0.85, otherwise pass.
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

