pub(super) fn append(doc: &mut String) {
    doc.push_str(
        r#"
#let story-force-review-page(title, chart1, cap1, chart2, cap2) = {
  page-title[#title]
  v(parse-pt(theme.section-gap))
  v(-4pt)
  two-chart-row(chart1, cap1, chart2, cap2)
}

#let drift-review-pair-page(title, data, chart-x, cap-x, chart-y, cap-y) = {
  let gov-is-x = data.x.governing-utilization >= data.y.governing-utilization
  let gov-dir = if gov-is-x { "X" } else { "Y" }
  let gov = if gov-is-x { data.x } else { data.y }
  let pass-str = if data.x.pass and data.y.pass { "PASS" } else { "FAIL" }
  page-title[#title]
  v(parse-pt(theme.section-gap))
  two-chart-row(chart-x, cap-x, chart-y, cap-y)
  governing-summary(
    stack(
      spacing: 3pt,
      section-label[Governing Direction: #gov-dir | Case: #gov.governing-case | Story: #gov.governing-story],
      body-note[Demand Ratio: #calc.round(gov.governing-demand-ratio, digits: 3) | Allowable Ratio: #calc.round(gov.allowable-ratio, digits: 3)],
    ),
    pass-str,
  )
}

#let drift-verify-pair-page(title, data) = {
  let side-by-side = data.x.groups.len() <= 4 and data.y.groups.len() <= 4
  page-title[#title]
  v(parse-pt(theme.section-gap))
  if side-by-side {
    with-divider(
      block(breakable: false)[#stack(spacing: 4pt, section-label[X Direction], drift-table(data.x))],
      block(breakable: false)[#stack(spacing: 4pt, section-label[Y Direction], drift-table(data.y))],
    )
  } else {
    stack(
      spacing: 8pt,
      section-label[X Direction],
      drift-table(data.x),
      section-label[Y Direction],
      drift-table(data.y),
    )
  }
}

#let displacement-review-pair-page(title, data, chart-x, cap-x, chart-y, cap-y) = {
  let gov-is-x = data.x.governing-utilization >= data.y.governing-utilization
  let gov-dir = if gov-is-x { "X" } else { "Y" }
  let gov = if gov-is-x { data.x } else { data.y }
  let pass-str = if data.x.pass and data.y.pass { "PASS" } else { "FAIL" }
  page-title[#title]
  v(parse-pt(theme.section-gap))
  two-chart-row(chart-x, cap-x, chart-y, cap-y)
  governing-summary(
    stack(
      spacing: 3pt,
      section-label[Governing Direction: #gov-dir | Case: #gov.governing-case | Story: #gov.governing-story],
      body-note[Demand (in): #calc.round(gov.governing-utilization * gov.governing-limit-in, digits: 3) | Limit (in): #calc.round(gov.governing-limit-in, digits: 3)],
      body-note[Basis: per-level limit = level elevation / configured ratio divisor.],
    ),
    pass-str,
  )
}

#let displacement-verify-pair-page(title, data) = {
  page-title[#title]
  v(parse-pt(theme.section-gap))
  grid(
    columns: eval(theme.two-col-ratio, mode: "code"),
    gutter: parse-pt(theme.grid-gutter),
    [#block(breakable: false)[#stack(spacing: 4pt, section-label[X Direction], displacement-table(data.x))]],
    [#block(breakable: false)[#stack(spacing: 4pt, section-label[Y Direction], displacement-table(data.y))]],
  )
}

#let torsion-dir-table(data-node) = {
  let rows = if data-node.rows.len() > 0 {
    data-node.rows.map(row => (
      row.story,
      row.case,
      row.joint-a,
      row.joint-b,
      str(row.governing-step),
      str(calc.round(row.drift-a, digits: 3)),
      str(calc.round(row.drift-b, digits: 3)),
      str(calc.round(row.delta-max, digits: 3)),
      str(calc.round(row.delta-avg, digits: 3)),
      ratio-cell(row.ratio-color-value, row.ratio-color-scale-kind),
    )).flatten()
  } else {
    ("-", "-", "-", "-", "-", "-", "-", "-", "-", "-")
  }
  table(
    columns: (1fr, 1fr, 1fr, 1fr, 0.65fr, 0.8fr, 0.8fr, 0.9fr, 0.9fr, 0.8fr),
    fill: (x, y) => if y == 0 { luma(220) } else { row-fill("", y) },
    align: (x, y) => if x >= 4 { right } else { left },
    repeating-header(
      10,
      ..("Story", "Case", "Joint A", "Joint B", "Step", "Drift A", "Drift B", "DeltaMax", "DeltaAvg", "Ratio")
        .map(h => table.cell(fill: luma(220))[#h])
    ),
    ..rows,
  )
}

#let torsion-review-pair-page(title, data, chart-x, chart-y) = {
  let pass-str = if data.x.has-type-b or data.y.has-type-b { "FAIL" } else { "PASS" }
  page-title[#title]
  v(parse-pt(theme.section-gap))
  two-chart-row(chart-x, [Torsional Ratio — X Direction], chart-y, [Torsional Ratio — Y Direction])
  governing-summary(
    stack(
      spacing: 3pt,
      section-label[X Governing: #data.x.governing-story | #data.x.governing-case | Ratio #calc.round(data.x.governing-ratio, digits: 3)],
      section-label[Y Governing: #data.y.governing-story | #data.y.governing-case | Ratio #calc.round(data.y.governing-ratio, digits: 3)],
    ),
    pass-str,
  )
}

#let torsion-verify-pair-page(title, data) = {
  page-title[#title]
  v(parse-pt(theme.section-gap))
  with-divider(
    [#stack(spacing: 4pt, section-label[X Direction], torsion-dir-table(data.x))],
    [#stack(spacing: 4pt, section-label[Y Direction], torsion-dir-table(data.y))],
  )
}

#let pier-shear-dir-table(rows) = {
  let body = if rows.len() > 0 {
    rows.map(row => (
      row.story,
      row.pier,
      str(calc.round(row.limit, digits: 3)),
      ratio-cell(row.ratio-color-value, row.ratio-color-scale-kind),
      str(calc.round(row.stress-psi, digits: 3)),
      str(calc.round(row.ve-kip, digits: 3)),
      str(calc.round(row.acw-in2, digits: 3)),
      str(calc.round(row.fc-psi, digits: 3)),
    )).flatten()
  } else {
    ("-", "-", "-", "-", "-", "-", "-", "-")
  }
  table(
    columns: (0.9fr, 0.8fr, 0.8fr, 0.85fr, 0.85fr, 0.85fr, 0.95fr, 0.85fr),
    fill: (x, y) => if y == 0 { luma(220) } else { row-fill("", y) },
    align: (x, y) => if x <= 1 { left } else { right },
    repeating-header(
      8,
      table.cell(fill: luma(220))[Story],
      table.cell(fill: luma(220))[Pier],
      table.cell(fill: luma(220))[Limit],
      table.cell(fill: luma(220))[Stress Ratio],
      table.cell(fill: luma(220))[Stress (psi)],
      table.cell(fill: luma(220))[Ve (kip)],
      table.cell(fill: luma(220))[Acw (in^2)],
      table.cell(fill: luma(220))[fc (psi)],
    ),
    ..body,
  )
}

#let pier-shear-average-dir-table(rows) = {
  let body = if rows.len() > 0 {
    rows.map(row => (
      row.story,
      str(calc.round(row.limit, digits: 3)),
      str(calc.round(row.avg-stress-psi, digits: 3)),
      str(calc.round(row.sum-area-in2, digits: 3)),
      str(calc.round(row.sum-shear-kip, digits: 3)),
      ratio-cell(row.ratio-color-value, row.ratio-color-scale-kind),
    )).flatten()
  } else {
    ("-", "-", "-", "-", "-", "-")
  }
  table(
    columns: (0.95fr, 0.75fr, 1fr, 1fr, 1fr, 0.8fr),
    fill: (x, y) => if y == 0 { luma(220) } else { row-fill("", y) },
    align: (x, y) => if x == 0 { left } else { right },
    table.header(repeat: true, [Story], [Limit], [Avg Stress (psi)], [Sum Area (in^2)], [Sum Shear (kip)], [Avg Ratio]),
    ..body,
  )
}

#let pier-shear-review-pair-page(title, data, chart-x, cap-x, chart-y, cap-y) = {
  if data.supported == false {
    page-title[#title]
    v(parse-pt(theme.section-gap))
    section-label[This check is currently unavailable for the configured code.]
    body-note[#data.support-note]
  } else {
    let pass-str = if data.pass { "PASS" } else { "FAIL" }
    page-title[#title]
    v(parse-pt(theme.section-gap))
    two-chart-row(chart-x, cap-x, chart-y, cap-y)
    governing-summary(
      stack(
        spacing: 3pt,
        section-label[Pier Shear Stress Ratio],
        body-note[Limit line: #calc.round(data.limit-individual-ratio, digits: 3) | Observed max ratio: #calc.round(data.max-individual-ratio, digits: 3)],
      ),
      pass-str,
    )
  }
}

#let pier-shear-verify-pair-page(title, data) = {
  if data.supported == false {
    page-title[#title]
    v(parse-pt(theme.section-gap))
    section-label[This check is currently unavailable for the configured code.]
    body-note[#data.support-note]
  } else {
    page-title[#title]
    v(parse-pt(theme.section-gap))
    with-divider(
      [#stack(spacing: 4pt, section-label[X Wall Direction], pier-shear-table(data.x-matrix))],
      [#stack(spacing: 4pt, section-label[Y Wall Direction], pier-shear-table(data.y-matrix))],
    )
  }
}

#let pier-shear-average-review-page(title, data-file, chart-file) = {
  let data = json(data-file)
  if data.supported == false {
    page-title[#title]
    v(parse-pt(theme.section-gap))
    section-label[This check is currently unavailable for the configured code.]
    body-note[#data.support-note]
  } else {
    let pass-str = if data.pass { "PASS" } else { "FAIL" }
    page-title[#title]
    v(parse-pt(theme.section-gap))
    block(breakable: false)[
      #align(center)[
        #ext-figure(chart-file, [Average Shear Ratio (X/Y)], parse-in(theme.chart-single-h))
      ]
      #v(6pt)
      #body-note[Average limit line: #calc.round(data.limit-average-ratio, digits: 3) | Observed max average ratio: #calc.round(data.max-average-ratio, digits: 3)]
      #v(2pt)
      #section-label[Status: #status-text(pass-str)]
    ]
  }
}

#let pier-shear-average-verify-page(title, data-file) = {
  let data = json(data-file)
  if data.supported == false {
    page-title[#title]
    v(parse-pt(theme.section-gap))
    section-label[This check is currently unavailable for the configured code.]
    body-note[#data.support-note]
  } else {
    page-title[#title]
    v(parse-pt(theme.section-gap))
    with-divider(
      [#stack(spacing: 4pt, section-label[X Average], pier-shear-average-dir-table(data.x-average-rows))],
      [#stack(spacing: 4pt, section-label[Y Average], pier-shear-average-dir-table(data.y-average-rows))],
    )
  }
}
"#,
    );
}
