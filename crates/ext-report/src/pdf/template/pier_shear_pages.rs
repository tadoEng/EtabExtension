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
"#,
    );
}
