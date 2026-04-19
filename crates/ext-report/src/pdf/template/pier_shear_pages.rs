pub(super) fn append(doc: &mut String) {
    doc.push_str(
        r#"
#let pier-shear-table(data) = {
  table(
    columns: data.piers.len() + 1,
    fill: (x, y) => if y == 0 { luma(220) } else { row-fill("", y) },
    align: (x, y) => if x == 0 { left } else { right },
    table.header(repeat: true, [Level], ..data.piers.map(p => [#p])),
    ..range(data.levels.len()).map(i => {
      let row = data.matrix-ratio.at(i, default: ())
      (
        data.levels.at(i, default: "-"),
        ..range(data.piers.len()).map(j => {
          let value = row.at(j, default: none)
          if value == none {
            "-"
          } else {
            table.cell(
              align: right,
              fill: ratio-fill(value, data.individual-ratio-scale-kind),
            )[#str(calc.round(value, digits: 3))]
          }
        }),
      )
    }).flatten(),
  )
}

#let pier-shear-avg-table-rows(rows) = {
  table(
    columns: (1fr, 1fr, 1fr, 1fr),
    fill: (x, y) => if y == 0 { luma(220) } else { row-fill("", y) },
    align: (x, y) => if x == 0 { left } else { right },
    table.header(repeat: true, [Story], [Limit Avg], [X Avg Ratio], [Y Avg Ratio]),
    ..rows.map(row => (
      row.story,
      str(calc.round(row.limit-average, digits: 3)),
      if row.x-average-stress-ratio == none {
        "-"
      } else {
        table.cell(
          align: right,
          fill: ratio-fill(row.x-color-value, row.x-color-scale-kind),
        )[#str(calc.round(row.x-average-stress-ratio, digits: 3))]
      },
      if row.y-average-stress-ratio == none {
        "-"
      } else {
        table.cell(
          align: right,
          fill: ratio-fill(row.y-color-value, row.y-color-scale-kind),
        )[#str(calc.round(row.y-average-stress-ratio, digits: 3))]
      },
    )).flatten(),
  )
}

#let pier-shear-avg-table(data) = {
  if data.average-rows.len() > 22 {
    let left-rows = data.average-rows.slice(0, 22)
    let right-rows = data.average-rows.slice(22)
    grid(
      columns: (1fr, 1fr),
      gutter: 10pt,
      [#pier-shear-avg-table-rows(left-rows)],
      [#pier-shear-avg-table-rows(right-rows)],
    )
  } else {
    pier-shear-avg-table-rows(data.average-rows)
  }
}

#let pier-shear-individual-page(title, data-file, chart-file) = {
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
    let table-body = [#align(top)[
      #section-label[Levels: #data.levels.len() | Piers: #data.piers.len() | Quantity: stress ratio]
      #v(4pt)
      #pier-shear-table(data)
    ]]
    let chart-body = [#stack(
      spacing: 0pt,
      align(center)[
        #ext-figure(chart-file, [Pier Shear Individual Stress Ratio Trend By Story], parse-in(theme.chart-with-table-normal-h))
      ],
      v(1fr),
      align(right)[
        #stack(
          spacing: 2pt,
          body-note[Limit basis: individual ratio <= #calc.round(data.limit-individual-ratio, digits: 3)],
          body-note[Observed max individual ratio: #calc.round(data.max-individual-ratio, digits: 3)],
          section-label[Status: #status-text(pass-str)],
        )
      ],
    )]
    chart-table-layout(table-body, chart-body)
  }
}

#let pier-shear-average-page(title, data-file, chart-file) = {
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
    let table-body = [#align(top)[
      #section-label[Stories: #data.average-rows.len() | Quantity: average stress ratio]
      #v(4pt)
      #pier-shear-avg-table(data)
    ]]
    let chart-body = [#stack(
      spacing: 0pt,
      align(center)[
        #ext-figure(chart-file, [Pier Shear Average Stress Ratio Trend By Story], parse-in(theme.chart-with-table-normal-h))
      ],
      v(1fr),
      align(right)[
        #stack(
          spacing: 2pt,
          body-note[Limit basis: average ratio <= #calc.round(data.limit-average-ratio, digits: 3)],
          body-note[Observed max average ratio: #calc.round(data.max-average-ratio, digits: 3)],
          section-label[Status: #status-text(pass-str)],
        )
      ],
    )]
    chart-table-layout(table-body, chart-body)
  }
}

#let pier-axial-assumptions(data-file) = {
  let data = json(data-file)
  let pass-str = if data.pass { "PASS" } else { "FAIL" }
  page-title[Pier Axial Preliminary Screening]
  v(parse-pt(theme.section-gap))
  section-label[Conservative Capacity Basis]
  v(8pt)
  body-note[Nominal capacity uses Po = 0.85fcAg and phiPo = phi ** Po.]
  body-note[This section is a preliminary axial screening check, not a full wall/pier axial design check.]
  body-note[Rebar contribution is intentionally excluded from this preliminary axial check.]
  body-note[Fallback fc reuses the pier section material default when pier/story matching is unavailable.]
  body-note[Results are split by gravity, wind, and seismic categories.]
  v(1fr)
  align(right)[
    #stack(
      spacing: 2pt,
      body-note[Preliminary screening only. Final design check requires engineer review.],
      section-label[Status: #status-text(pass-str)],
    )
  ]
}
"#,
    );
}
