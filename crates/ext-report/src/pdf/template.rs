use ext_calc::output::CalcOutput;


pub fn build_typst_document(calc: &CalcOutput) -> String {
    let mut doc = String::new();

    // ── Global Typst Boilerplate & Layout Setup ─────────────────────────────────
    doc.push_str(
        r##"
#let project = json("project.json")
#let theme = json("theme.json")

#let parse-pt(s) = float(s.trim("pt")) * 1pt
#let parse-in(s) = float(s.trim("in")) * 72pt

#let is-executive = theme.layout-kind == "executive"

#let page-width = parse-in(theme.page-width)
#let page-height = parse-in(theme.page-height)
#let m-top = parse-in(theme.margin-top)
#let m-left = parse-in(theme.margin-left)
#let m-right = parse-in(theme.margin-right)
#let m-bottom = parse-in(theme.margin-bottom)
#let content-h = parse-in(theme.content-height)

#let border-w = page-width - m-left - m-right
#let border-h = page-height - m-top - m-bottom
#let tb-h = border-h - content-h

#set text(font: theme.body-font, size: parse-pt(theme.body-size))
#set par(justify: false)
#set figure(numbering: none, outlined: false)
#show heading: set block(sticky: true)

#let title-block() = {
  let cols = theme.title-block-columns.trim("(").trim(")").split(",").map(s => parse-in(s.trim()))
  table(
    columns: cols,
    stroke: 1pt + black,
    inset: parse-pt(theme.table-inset),

    [
      #align(center + horizon)[
        #stack(spacing: 0pt,
          text(size: 11pt, weight: "bold")[Thornton],
          text(size: 11pt, weight: "bold")[Tomasetti],
        )
      ]
    ],
    [
      #stack(spacing: 2pt,
        text(size: 5.5pt, fill: luma(110))[PROJECT],
        text(size: 8pt, weight: "bold")[#project.project-name],
        text(size: 5.5pt, fill: luma(110))[PROJECT NO.],
        text(size: 7.5pt)[#project.project-number],
      )
    ],
    [
      #stack(spacing: 2pt,
        text(size: 5.5pt, fill: luma(110))[DRAWING TITLE],
        text(size: 8.5pt, weight: "bold")[#project.subject],
      )
    ],
    [
      #stack(spacing: 2pt,
        text(size: 5.5pt, fill: luma(110))[REFERENCE],
        text(size: 7.5pt)[#project.reference],
        text(size: 5.5pt, fill: luma(110))[REVISION],
        text(size: 8pt, weight: "bold")[#project.revision],
      )
    ],
    [
      #stack(spacing: 2pt,
        text(size: 5.5pt, fill: luma(110))[DRAWN BY],
        text(size: 8pt, weight: "bold")[#project.engineer],
        text(size: 5.5pt, fill: luma(110))[CHECKED BY],
        text(size: 8pt, weight: "bold")[#project.checker],
      )
    ],
    [
      #stack(spacing: 2pt,
        text(size: 5.5pt, fill: luma(110))[DATE],
        text(size: 7.5pt)[#project.date],
        text(size: 5.5pt, fill: luma(110))[SCALE / SHEET],
        text(size: 8pt)[#project.scale],
        text(size: 14pt, weight: "bold")[#project.sheet-prefix#"-"#counter(page).display("01")],
      )
    ],
  )
}

#set page(
  width: page-width,
  height: page-height,
  margin: (
    top: m-top,
    left: m-left,
    right: m-right,
    bottom: if is-executive { m-bottom } else { m-bottom + tb-h },
  ),
  header: if is-executive {
    context {
      set text(font: theme.body-font, size: 9pt, fill: luma(80))
      stack(
        spacing: 6pt,
        grid(
          columns: (1fr, auto),
          align(left)[*Thornton Tomasetti* | #project.project-name (#project.project-number)],
          align(right)[#project.subject],
        ),
        line(length: 100%, stroke: 0.5pt + luma(180)),
      )
    }
  } else {
    none
  },
  footer: if is-executive {
    context {
      set text(font: theme.body-font, size: 9pt, fill: luma(80))
      stack(
        spacing: 6pt,
        line(length: 100%, stroke: 0.5pt + luma(180)),
        grid(
          columns: (1fr, auto),
          align(left)[Date: #project.date | By: #project.engineer | Ref: #project.reference],
          align(right)[Page #counter(page).display("1 of 1", both: true)],
        ),
      )
    }
  } else {
    none
  },
  background: if is-executive {
    none
  } else {
    context {
      place(
        top + left,
        dx: m-left,
        dy: m-top,
        rect(
          width: border-w,
          height: border-h,
          stroke: 1.2pt + black,
          inset: 0pt,
          outset: 0pt,
          [
            #place(bottom + left)[
              #box(width: border-w, height: tb-h)[
                #title-block()
              ]
            ]
          ]
        )
      )
    }
  },
)

#set table(stroke: 0.5pt + luma(180), inset: parse-pt(theme.table-inset))
#show table.cell.where(y: 0): set text(weight: "bold", size: parse-pt(theme.label-size))

#let tag-fill(tag) = {
  if tag == "ux_threshold" { rgb("#cfe2ff") }
  else if tag == "uy_threshold" { rgb("#fff3cd") }
  else if tag == "ux_uy_threshold" { rgb("#d1c4e9") }
  else if tag == "high" { rgb("#e8f5e9") }
  else if tag == "pass" { rgb("#d4edda") }
  else if tag == "warn" { rgb("#fff3cd") }
  else if tag == "fail" { rgb("#f8d7da") }
  else { none }
}

#let row-fill(tag, row-idx) = {
  let explicit = tag-fill(tag)
  if explicit != none { explicit }
  else if calc.odd(row-idx) { luma(248) }
  else { none }
}
"##,
    );

    // ── Layout & Cell Helpers ───────────────────────────────────────────────────
    doc.push_str(
        r#"
#let styled-cell(fill-tag, row-idx, align-dir, content) = {
  table.cell(fill: row-fill(fill-tag, row-idx), align: align-dir)[#content]
}

#let chart-table-layout(table-body, chart-body, emphasized: false) = {
  if is-executive {
    stack(
      spacing: parse-pt(theme.grid-gutter),
      chart-body,
      table-body,
    )
  } else {
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
}

#let single-chart-page(title, chart1, chart1-caption) = {
  text(size: parse-pt(theme.title-size), weight: "bold")[#title]
  v(parse-pt(theme.section-gap))
  align(center)[
    #figure(
      image(chart1, height: parse-in(theme.chart-single-h)),
      caption: text(size: parse-pt(theme.caption-size))[#chart1-caption],
    )
  ]
}

#let two-charts-page(title, chart1, chart1-caption, chart2, chart2-caption) = {
  text(size: parse-pt(theme.title-size), weight: "bold")[#title]
  v(parse-pt(theme.section-gap))
  if is-executive {
    stack(
      spacing: parse-pt(theme.grid-gutter),
      figure(
        image(chart1, height: parse-in(theme.chart-two-col-h)),
        caption: text(size: parse-pt(theme.caption-size))[#chart1-caption],
      ),
      figure(
        image(chart2, height: parse-in(theme.chart-two-col-h)),
        caption: text(size: parse-pt(theme.caption-size))[#chart2-caption],
      ),
    )
  } else {
    grid(
      columns: eval(theme.two-col-ratio, mode: "code"),
      gutter: parse-pt(theme.grid-gutter),
      [#figure(
        image(chart1, height: parse-in(theme.chart-two-col-h)),
        caption: text(size: parse-pt(theme.caption-size))[#chart1-caption],
      )],
      [#figure(
        image(chart2, height: parse-in(theme.chart-two-col-h)),
        caption: text(size: parse-pt(theme.caption-size))[#chart2-caption],
      )],
    )
  }
}
"#,
    );

    // ── Section Logic ───────────────────────────────────────────────────────────
    doc.push_str(
        r#"
#let summary-page() = {
  let data = json("summary.json")
  text(size: parse-pt(theme.title-size), weight: "bold")[#project.project-name]
  text(size: parse-pt(theme.label-size), fill: luma(90))[#project.subject]
  v(10pt)
  grid(
    columns: (1fr, 1fr),
    gutter: 16pt,
    [
      #stack(
        spacing: 3pt,
        [*Reference:* #project.reference],
        [*Project No.:* #project.project-number],
        [*Revision:* #project.revision],
        [*Branch / Version:* #data.branch / #data.version-id],
      )
    ],
    [
      #stack(
        spacing: 3pt,
        [*Engineer:* #project.engineer],
        [*Checker:* #project.checker],
        [*Date:* #project.date],
        [*Status:* #data.overall-status],
      )
    ],
  )
  v(parse-pt(theme.section-gap))
  text(size: parse-pt(theme.title-size), weight: "bold")[Report Summary]
  v(6pt)
  for line in data.lines [
    - #line.key (#line.status) #line.message
  ]
}

#let modal-page() = {
  let data = json("modal.json")
  text(size: parse-pt(theme.title-size), weight: "bold")[Modal Participation]
  v(parse-pt(theme.section-gap))
  text(size: parse-pt(theme.label-size), weight: "bold")[Mass participation threshold = #(data.threshold * 100.0)%]
  v(4pt)
  table(
    columns: (1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr),
    fill: (x, y) => if y == 0 { luma(220) } else { row-fill(data.annotations.at(y - 1, default: ""), y) },
    align: (x, y) => if x >= 1 { right } else { left },
    table.header(
      ..("Mode", "Period", "UX", "UY", "Sum UX", "Sum UY", "Highlight")
        .map(h => table.cell(fill: luma(220))[#h])
    ),
    ..range(data.rows.len()).map(i => {
      let row = data.rows.at(i)
      let tag = data.annotations.at(i, default: "")
      let highlight-label = if tag == "ux_threshold" { "UX threshold" }
                            else if tag == "uy_threshold" { "UY threshold" }
                            else if tag == "ux_uy_threshold" { "UX/UY threshold" }
                            else { "" }
      (
        str(row.mode),
        str(calc.round(row.period, digits: 3)),
        str(calc.round(row.ux * 100.0, digits: 1)) + "%",
        str(calc.round(row.uy * 100.0, digits: 1)) + "%",
        str(calc.round(row.sum-ux * 100.0, digits: 1)) + "%",
        str(calc.round(row.sum-uy * 100.0, digits: 1)) + "%",
        highlight-label,
      )
    }).flatten(),
  )
}

#let base-reactions-table(data) = {
  table(
    columns: (1fr, 1fr, 1fr, 1fr, 1fr),
    fill: (x, y) => if y == 0 { luma(220) } else { row-fill("", y) },
    align: (x, y) => if x >= 2 { right } else { left },
    table.header(
      ..("Load Case", "Type", "Fx (kip)", "Fy (kip)", "Fz (kip)")
        .map(h => table.cell(fill: luma(220))[#h])
    ),
    ..data.rows.map(row => (
      row.output-case,
      row.case-type,
      str(calc.round(row.fx-kip, digits: 1)),
      str(calc.round(row.fy-kip, digits: 1)),
      str(calc.round(row.fz-kip, digits: 1)),
    )).flatten(),
  )
}

#let base-reactions-page() = {
  let data = json("base_reactions.json")
  text(size: parse-pt(theme.title-size), weight: "bold")[Base Reaction Review]
  v(parse-pt(theme.section-gap))
  let table-body = [#align(top)[
    #text(size: parse-pt(theme.label-size), weight: "bold")[All extracted base reaction load cases. Gravity pie includes configured gravity cases.]
    #v(4pt)
    #base-reactions-table(data)
  ]]
  let chart-body = [#align(center)[
    #figure(
      image("images/base_reactions.svg", height: parse-in(theme.chart-with-table-normal-h)),
      caption: text(size: parse-pt(theme.caption-size))[Base Reactions (kip)],
    )
  ]]
  chart-table-layout(table-body, chart-body)
}

#let story-forces-page(title, chart1, chart2) = {
  two-charts-page(title, chart1, "Shear (kip)", chart2, "Moment (kip-ft)")
}

#let drift-table(data-node) = {
  table(
    columns: (1fr, 1fr, 1fr, 1fr, 1fr),
    fill: (x, y) => if y == 0 { luma(220) } else { row-fill(data-node.annotations.at(y - 1, default: ""), y) },
    align: (x, y) => if x >= 2 { right } else { left },
    table.header(
      ..("Story", "Case", "Demand (ratio)", "Limit (ratio)", "DCR")
        .map(h => table.cell(fill: luma(220))[#h])
    ),
    ..data-node.rows.map(row => (
      row.story,
      row.output-case,
      str(calc.round(row.drift-ratio, digits: 5)),
      str(calc.round(data-node.allowable-ratio, digits: 5)),
      str(calc.round(row.dcr, digits: 3)),
    )).flatten(),
  )
}

#let drift-dir-page(title, data-node, chart-file) = {
  text(size: parse-pt(theme.title-size), weight: "bold")[#title]
  v(parse-pt(theme.section-gap))
  let pass-str = if data-node.pass { "PASS" } else { "FAIL" }
  let table-body = [#align(top)[
    #text(size: parse-pt(theme.label-size), weight: "bold")[
      Governing: #data-node.governing-story #data-node.governing-direction #data-node.governing-case (#pass-str)
    ]
    #v(4pt)
    #drift-table(data-node)
  ]]
  let chart-body = [#align(center)[
    #figure(
      image(chart-file, height: parse-in(theme.chart-with-table-chart-h)),
      caption: text(size: parse-pt(theme.caption-size))[Drift Envelope],
    )
  ]]
  chart-table-layout(table-body, chart-body, emphasized: true)
}

#let displacement-table(data-node) = {
  table(
    columns: (1fr, 1fr, 1fr, 1fr, 1fr),
    fill: (x, y) => if y == 0 { luma(220) } else { row-fill(data-node.annotations.at(y - 1, default: ""), y) },
    align: (x, y) => if x >= 2 { right } else { left },
    table.header(
      ..("Story", "Case", "Demand (in)", "Limit (in)", "DCR")
        .map(h => table.cell(fill: luma(220))[#h])
    ),
    ..data-node.rows.map(row => (
      row.story,
      row.output-case,
      str(calc.round(row.demand-in, digits: 4)),
      str(calc.round(data-node.limit-in, digits: 4)),
      str(calc.round(row.dcr, digits: 3)),
    )).flatten(),
  )
}

#let displacement-dir-page(title, data-node, chart-file) = {
  text(size: parse-pt(theme.title-size), weight: "bold")[#title]
  v(parse-pt(theme.section-gap))
  let pass-str = if data-node.pass { "PASS" } else { "FAIL" }
  let table-body = [#align(top)[
    #text(size: parse-pt(theme.label-size), weight: "bold")[
      Governing: #data-node.governing-story #data-node.governing-direction #data-node.governing-case (#pass-str)
    ]
    #v(4pt)
    #displacement-table(data-node)
  ]]
  let chart-body = [#align(center)[
    #figure(
      image(chart-file, height: parse-in(theme.chart-with-table-chart-h)),
      caption: text(size: parse-pt(theme.caption-size))[Displacement Envelope],
    )
  ]]
  chart-table-layout(table-body, chart-body, emphasized: true)
}

#let torsional-dir-page(title, data-node) = {
  text(size: parse-pt(theme.title-size), weight: "bold")[#title]
  v(parse-pt(theme.section-gap))
  let warn-str = if data-node.has-type-b { "TYPE B IRREGULARITY" } else if data-node.has-type-a { "Type A irregularity" } else { "No irregularity" }
  text(size: parse-pt(theme.label-size), weight: "bold")[
    Governing story: #data-node.governing-story | Case: #data-node.governing-case | Max ratio: #calc.round(data-node.max-ratio, digits: 3) | #warn-str
  ]
  v(4pt)
  table(
    columns: (1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr),
    fill: (x, y) => if y == 0 { luma(220) } else { row-fill(data-node.annotations.at(y - 1, default: ""), y) },
    align: (x, y) => if x >= 4 { right } else { left },
    table.header(
      ..("Story", "Case", "Joint A", "Joint B", "Ratio", "Type A (>1.2)", "Type B (>1.4)", "Ax", "Ecc (ft)")
        .map(h => table.cell(fill: luma(220))[#h])
    ),
    ..data-node.rows.map(row => (
      row.story,
      row.case,
      row.joint-a,
      row.joint-b,
      str(calc.round(row.ratio, digits: 3)),
      if row.is-type-a { "Type A" } else { "-" },
      if row.is-type-b { "Type B" } else { "-" },
      str(calc.round(row.ax, digits: 2)),
      str(calc.round(row.ecc-ft, digits: 2)),
    )).flatten(),
  )
}
"#,
    );

    doc.push_str(
        r#"
#let pier-shear-page(title, data-file, chart-file) = {
  let data = json(data-file)
  let pass-str = if data.pass { "PASS" } else { "FAIL" }
  text(size: parse-pt(theme.title-size), weight: "bold")[#title]
  v(parse-pt(theme.section-gap))
  align(center)[
    #figure(
      image(chart-file, height: parse-in(theme.chart-with-table-normal-h)),
      caption: text(size: parse-pt(theme.caption-size))[Pier Shear Envelope],
    )
  ]
  v(8pt)
  text(size: parse-pt(theme.label-size), weight: "bold")[Rows: #data.rows.len() | Passed: #pass-str]
}

#let pier-axial-assumptions(data-file) = {
  let data = json(data-file)
  let pass-str = if data.pass { "PASS" } else { "FAIL" }
  text(size: parse-pt(theme.title-size), weight: "bold")[Pier Axial Assumptions]
  v(parse-pt(theme.section-gap))
  text(weight: "bold")[Conservative Capacity Basis]
  v(8pt)
  text(size: parse-pt(theme.body-size))[Nominal capacity uses Po = 0.85fcAg and phiPo = phi ** Po.]
  text(size: parse-pt(theme.body-size))[Rebar contribution is intentionally excluded from this preliminary axial check.]
  text(size: parse-pt(theme.body-size))[Fallback fc reuses the pier section material default when pier/story matching is unavailable.]
  text(size: parse-pt(theme.body-size))[Results are split by gravity, wind, and seismic categories.]
  v(16pt)
  text(weight: "bold")[Overall Pass: #pass-str]
}
"#,
    );

    // ── Generate Sequence of Section Calls ──────────────────────────────────────
    doc.push_str("\n// ── Document Sequence ────────────────────────────────\n");
    doc.push_str("#summary-page()\n\n");

    if calc.modal.is_some() {
        doc.push_str("#pagebreak()\n#modal-page()\n\n");
    }

    if calc.base_reactions.is_some() {
        doc.push_str("#pagebreak()\n#base-reactions-page()\n\n");
    }

    if calc.story_forces.is_some() {
        doc.push_str("#pagebreak()\n#story-forces-page([Story Forces — X Direction], \"images/story_force_vx.svg\", \"images/story_force_my.svg\")\n\n");
        doc.push_str("#pagebreak()\n#story-forces-page([Story Forces — Y Direction], \"images/story_force_vy.svg\", \"images/story_force_mx.svg\")\n\n");
    }

    if calc.drift_wind.is_some() {
        doc.push_str("#pagebreak()\n#let dw = json(\"drift_wind.json\")\n#drift-dir-page([Wind Drift Review (X)], dw.x, \"images/drift_wind_x.svg\")\n\n");
        doc.push_str("#pagebreak()\n#drift-dir-page([Wind Drift Review (Y)], dw.y, \"images/drift_wind_y.svg\")\n\n");
    }

    if calc.drift_seismic.is_some() {
        doc.push_str("#pagebreak()\n#let ds = json(\"drift_seismic.json\")\n#drift-dir-page([Seismic Drift Review (X)], ds.x, \"images/drift_seismic_x.svg\")\n\n");
        doc.push_str("#pagebreak()\n#drift-dir-page([Seismic Drift Review (Y)], ds.y, \"images/drift_seismic_y.svg\")\n\n");
    }

    if calc.displacement_wind.is_some() {
        doc.push_str("#pagebreak()\n#let dpw = json(\"displacement_wind.json\")\n#displacement-dir-page([Wind Displacement Review (X)], dpw.x, \"images/displacement_wind_x.svg\")\n\n");
        doc.push_str("#pagebreak()\n#displacement-dir-page([Wind Displacement Review (Y)], dpw.y, \"images/displacement_wind_y.svg\")\n\n");
    }

    if let Some(torsional) = calc.torsional.as_ref() {
        let has_x = !torsional.x.rows.is_empty();
        let has_y = !torsional.y.rows.is_empty();
        if has_x || has_y {
            doc.push_str("#let tor = json(\"torsional.json\")\n");
        }
        if has_x {
            doc.push_str(
                "#pagebreak()\n#torsional-dir-page([Torsional Irregularity - X Direction], tor.x)\n\n",
            );
        }
        if has_y {
            doc.push_str(
                "#pagebreak()\n#torsional-dir-page([Torsional Irregularity - Y Direction], tor.y)\n\n",
            );
        }
    }

    if calc.pier_shear_stress_wind.is_some() {
        doc.push_str("#pagebreak()\n#pier-shear-page([Pier Shear Wind Review], \"pier_shear_wind.json\", \"images/pier_shear_stress_wind.svg\")\n\n");
    }

    if calc.pier_shear_stress_seismic.is_some() {
        doc.push_str("#pagebreak()\n#pier-shear-page([Pier Shear Seismic Review], \"pier_shear_seismic.json\", \"images/pier_shear_stress_seismic.svg\")\n\n");
    }

    if calc.pier_axial_stress.is_some() {
        doc.push_str("#pagebreak()\n#single-chart-page([Pier Axial - Gravity], \"images/pier_axial_gravity.svg\", \"Pier Axial - Gravity\")\n\n");
        doc.push_str("#pagebreak()\n#single-chart-page([Pier Axial - Wind], \"images/pier_axial_wind.svg\", \"Pier Axial - Wind\")\n\n");
        doc.push_str("#pagebreak()\n#single-chart-page([Pier Axial - Seismic], \"images/pier_axial_seismic.svg\", \"Pier Axial - Seismic\")\n\n");
        doc.push_str("#pagebreak()\n#pier-axial-assumptions(\"pier_axial_stress.json\")\n\n");
    }

    doc
}
