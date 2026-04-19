pub const INCLUDE_CALC_PROCEDURE_PAGE: bool = true;

pub fn append_definitions(doc: &mut String) {
    if !INCLUDE_CALC_PROCEDURE_PAGE {
        return;
    }

    doc.push_str(
        r#"
#let given-table(pairs) = {
  block(
    stroke: 0.5pt + luma(180),
    radius: 2pt,
    clip: true,
    width: 100%,
  )[
    #table(
      columns: (auto, 1fr),
      stroke: (x, y) => if y < pairs.len() - 1 {
        (bottom: 0.3pt + luma(210))
      } else { none },
      inset: (x: 8pt, y: 4pt),
      fill: (x, y) => if x == 0 { luma(242) } else { white },
      ..pairs.map(pair => (
        section-label[#pair.at(0)],
        body-note[#pair.at(1)],
      )).flatten(),
    )
  ]
}

#let calc-step(n, formula, substitution, result) = {
  v(4pt)
  grid(
    columns: (2em, 1fr),
    gutter: 0pt,
    align(top)[#section-label[#n.]],
    stack(
      spacing: 3pt,
      ref-note[#formula],
      pad(left: 8pt)[#body-note[= #substitution]],
      pad(left: 8pt)[
        #box(
          inset: (x: 8pt, y: 4pt),
          radius: 2pt,
          fill: luma(225),
          stroke: 0.5pt + luma(180),
        )[
          #text(size: parse-pt(theme.label-size) + 2pt, weight: "bold")[= #result]
        ]
      ],
    ),
  )
}

#let calc-result(label, value, pass) = {
  v(6pt)
  let bg = if pass { rgb(212, 237, 218) } else { rgb(248, 215, 218) }
  let border = if pass { rgb(25, 135, 84) } else { rgb(220, 53, 69) }
  block(
    fill: bg,
    stroke: (left: 3pt + border),
    inset: (left: 12pt, right: 10pt, top: 7pt, bottom: 7pt),
    radius: (right: 3pt),
    width: 100%,
  )[
    #grid(
      columns: (1fr, auto, auto),
      column-gutter: 10pt,
      align: horizon,
      section-label[#label],
      text(size: parse-pt(theme.label-size) + 2pt, weight: "bold")[#value],
      box(
        fill: border,
        inset: (x: 8pt, y: 3pt),
        radius: 2pt,
      )[
        #text(weight: "bold", fill: white, size: parse-pt(theme.label-size))[
          #if pass { [PASS] } else { [FAIL] }
        ]
      ],
    )
  ]
}

#let torsion-worked-example() = {
  let tor = json("torsional.json")
  let ex = if tor.x.has-rows { tor.x } else if tor.y.has-rows { tor.y } else { none }

  page-title[Torsional Irregularity — Worked Example]
  v(6pt)
  ref-note[
    Reference: ASCE 7 Table 12.3-1 — Torsional Irregularity Type 1a / 1b.\
    Criterion: δ_max / δ_avg > 1.2 (Type A), > 1.4 (Type B).
  ]
  v(8pt)

  if ex == none {
    body-note[No torsional governing row available.]
  } else {
    stack(
      spacing: 3pt,
      line(length: 100%, stroke: 0.4pt + luma(200)),
      section-label[Given],
    )
    v(4pt)
    given-table((
      ("Story",     ex.governing-story),
      ("Load case", ex.governing-case),
      ("Joint A",   ex.governing-joint-a),
      ("Joint B",   ex.governing-joint-b),
      ("Step",      str(ex.governing-step)),
      ("Δ_A (in)",  str(calc.round(ex.governing-drift-a, digits: 4))),
      ("Δ_B (in)",  str(calc.round(ex.governing-drift-b, digits: 4))),
    ))
    v(8pt)

    stack(
      spacing: 3pt,
      line(length: 100%, stroke: 0.4pt + luma(200)),
      section-label[Procedure],
    )
    v(4pt)
    calc-step(
      1,
      [δ_max = max(|Δ_A|, |Δ_B|)],
      [max(#calc.round(ex.governing-drift-a, digits: 4), #calc.round(ex.governing-drift-b, digits: 4))],
      [#calc.round(ex.governing-delta-max, digits: 4) in],
    )
    calc-step(
      2,
      [δ_avg = (|Δ_A| + |Δ_B|) / 2],
      [(#calc.round(ex.governing-drift-a, digits: 4) + #calc.round(ex.governing-drift-b, digits: 4)) / 2],
      [#calc.round(ex.governing-delta-avg, digits: 4) in],
    )
    calc-step(
      3,
      [Ratio = δ_max / δ_avg],
      [#calc.round(ex.governing-delta-max, digits: 4) / #calc.round(ex.governing-delta-avg, digits: 4)],
      [#calc.round(ex.governing-ratio, digits: 4)],
    )

    calc-result(
      [Torsional Ratio  |  Classification: #ex.classification],
      [#calc.round(ex.governing-ratio, digits: 4)],
      not ex.has-type-b,
    )
  }
}

#let pier-shear-worked-example() = {
  let wind = json("pier_shear_wind.json")
  let seismic = json("pier_shear_seismic.json")

  let data = if wind.supported and wind.x-rows.len() > 0       { (wind,    wind.x-rows.at(0))    }
        else if wind.supported and wind.y-rows.len() > 0       { (wind,    wind.y-rows.at(0))    }
        else if seismic.supported and seismic.x-rows.len() > 0 { (seismic, seismic.x-rows.at(0)) }
        else if seismic.supported and seismic.y-rows.len() > 0 { (seismic, seismic.y-rows.at(0)) }
        else { none }

  page-title[Pier Shear Stress — Worked Example]
  v(6pt)
  ref-note[
    Reference: ACI 318-14 §18.10.4.\
    Formula: v_u = V_e / (φ_v × A_cw), stress ratio = v_u / √f'c.
  ]
  v(8pt)

  if data == none {
    body-note[No pier shear row available.]
  } else {
    let src = data.at(0)
    let row = data.at(1)
    let sqrt-fc = calc.sqrt(row.fc-psi)

    stack(
      spacing: 3pt,
      line(length: 100%, stroke: 0.4pt + luma(200)),
      section-label[Given],
    )
    v(4pt)
    given-table((
      ("Story",      row.story),
      ("Pier",       row.pier),
      ("Load combo", row.combo),
      ("V_e",        [#calc.round(row.ve-kip,  digits: 3) kip]),
      ("A_cw",       [#calc.round(row.acw-in2, digits: 3) in²]),
      ("f'c",        [#calc.round(row.fc-psi,  digits: 0) psi]),
      ("√f'c",       [#calc.round(sqrt-fc,     digits: 3)]),
      ("φ_v",        [#calc.round(src.phi-v,   digits: 2)]),
    ))
    v(8pt)

    stack(
      spacing: 3pt,
      line(length: 100%, stroke: 0.4pt + luma(200)),
      section-label[Procedure],
    )
    v(4pt)
    calc-step(
      1,
      [v_u = V_e × 1000 / (φ_v × A_cw)],
      [#calc.round(row.ve-kip, digits: 3) × 1000 / (#calc.round(src.phi-v, digits: 2) × #calc.round(row.acw-in2, digits: 3))],
      [#calc.round(row.stress-psi, digits: 3) psi],
    )
    calc-step(
      2,
      [Stress ratio = v_u / √f'c],
      [#calc.round(row.stress-psi, digits: 3) / #calc.round(sqrt-fc, digits: 3)],
      [#calc.round(row.stress-ratio, digits: 3)],
    )

    calc-result(
      [Stress Ratio  |  Limit: #calc.round(row.limit, digits: 3)],
      [#calc.round(row.stress-ratio, digits: 3)],
      row.stress-ratio <= row.limit,
    )
  }
}

#let calc-procedure-page() = {
  page-title[Calculation Trace — Governing Case Examples]
  v(parse-pt(theme.section-gap))
  body-note[
    The following hand-calculation traces reproduce the governing results directly from the model data.
    Values match the report tables to ±0.001 rounding.
  ]
  v(10pt)
  if is-executive {
    torsion-worked-example()
    v(16pt)
    pier-shear-worked-example()
  } else {
    with-divider(
      [#torsion-worked-example()],
      [#pier-shear-worked-example()],
    )
  }
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
