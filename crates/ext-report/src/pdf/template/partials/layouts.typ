#let chart-table-layout(table-body, chart-body, emphasized: false) = {
  let cols = if emphasized {
    eval(theme.chart-table-emphasized, mode: "code")
  } else {
    eval(theme.chart-table-normal, mode: "code")
  }
  grid(
    columns: cols,
    gutter: parse-pt(theme.grid-gutter),
    [#align(top)[#table-body]],
    [#align(center)[#chart-body]],
  )
}

#let single-chart-page(title, chart1, chart1-caption) = {
  page-title[#title]
  v(parse-pt(theme.section-gap))
  align(center)[
    #ext-figure(chart1, chart1-caption, parse-in(theme.chart-single-h))
  ]
}

#let two-chart-row(chart1, cap1, chart2, cap2) = {
  grid(
    columns: eval(theme.two-col-ratio, mode: "code"),
    gutter: parse-pt(theme.grid-gutter),
    ext-figure(chart1, cap1, parse-in(theme.chart-two-col-h)),
    ext-figure(chart2, cap2, parse-in(theme.chart-two-col-h)),
  )
}

#let with-divider(left-content, right-content) = {
  grid(
    columns: (1fr, 1pt, 1fr),
    column-gutter: parse-pt(theme.grid-gutter) / 2,
    left-content,
    pad(top: 8pt, bottom: 8pt)[
      #line(angle: 90deg, length: 100%, stroke: 0.35pt + luma(200))
    ],
    right-content,
  )
}

#let two-charts-page(title, chart1, chart1-caption, chart2, chart2-caption) = {
  page-title[#title]
  v(parse-pt(theme.section-gap))
  two-chart-row(chart1, chart1-caption, chart2, chart2-caption)
}
