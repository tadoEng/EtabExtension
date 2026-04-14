use ext_calc::output::CalcOutput;
use crate::pdf::procedures;


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
#let content-inset = parse-pt(theme.content-inset)

#let border-w = page-width - m-left - m-right
#let border-h = page-height - m-top - m-bottom
#let tb-h = border-h - content-h

#let text-m-top = if is-executive { m-top } else { m-top + content-inset }
#let text-m-left = if is-executive { m-left } else { m-left + content-inset }
#let text-m-right = if is-executive { m-right } else { m-right + content-inset }
#let text-m-bottom = if is-executive { m-bottom } else { m-bottom + tb-h + content-inset }

#set text(font: theme.body-font, size: parse-pt(theme.body-size))
#set par(justify: false)
#set figure(numbering: none, outlined: false)
#show heading: set block(sticky: true)

#let title-block(sheet) = {
  let cols = eval(theme.title-block-columns, mode: "code")
  table(
    columns: cols,
    rows: (tb-h,),
    stroke: 1.2pt + black,
    inset: 8pt,
    align: top + left,

    align(center + horizon)[
      #stack(
        spacing: 5pt,
        text(size: 15pt, weight: "bold")[Thornton],
        text(size: 15pt, weight: "bold")[Tomasetti],
      )
    ],
    stack(
      spacing: 4pt,
      text(size: 8pt, fill: luma(110))[PROJECT],
      text(size: 10pt, weight: "bold")[#project.project-name],
      v(4pt),
      text(size: 8pt, fill: luma(110))[PROJECT NO.],
      text(size: 10pt)[#project.project-number],
    ),
    stack(
      spacing: 4pt,
      text(size: 8pt, fill: luma(110))[DRAWING TITLE],
      text(size: 10pt, weight: "bold")[#project.subject],
    ),
    stack(
      spacing: 4pt,
      text(size: 5.5pt, fill: luma(110))[REFERENCE],
      text(size: 7.5pt)[#project.reference],
      v(4pt),
      text(size: 5.5pt, fill: luma(110))[REVISION],
      text(size: 8pt, weight: "bold")[#project.revision],
    ),
    stack(
      spacing: 4pt,
      text(size: 5.5pt, fill: luma(110))[DRAWN BY],
      text(size: 8pt, weight: "bold")[#project.engineer],
      v(4pt),
      text(size: 5.5pt, fill: luma(110))[CHECKED BY],
      text(size: 8pt, weight: "bold")[#project.checker],
    ),
    stack(
      spacing: 4pt,
      text(size: 5.5pt, fill: luma(110))[DATE],
      text(size: 7.5pt)[#project.date],
      v(4pt),
      text(size: 5.5pt, fill: luma(110))[SHEET],
      text(size: 14pt, weight: "bold")[#sheet],
    ),
  )
}

#set page(
  width: page-width,
  height: page-height,
  margin: (
    top: text-m-top,
    left: text-m-left,
    right: text-m-right,
    bottom: text-m-bottom,
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
    pad(
      top: m-top,
      bottom: m-bottom,
      left: m-left,
      right: m-right,
      align(top)[
        #stack(
          spacing: 0pt,
          rect(
            width: border-w,
            height: content-h,
            stroke: (
              top: 1.2pt + black,
              left: 1.2pt + black,
              right: 1.2pt + black,
              bottom: none,
            ),
          ),
          context {
            let sheet = project.sheet-prefix + "-" + str(counter(page).get().first())
            title-block(sheet)
          },
        )
      ],
    )
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
"#,
    );

    // ── Section Logic ───────────────────────────────────────────────────────────
    doc.push_str(
        r#"
#let cover-info-row(label, value) = {
  stack(
    spacing: 2pt,
    text(size: parse-pt(theme.label-size), fill: luma(100), weight: "bold")[#upper(label)],
    text(size: parse-pt(theme.body-size))[#value],
    line(length: 100%, stroke: 0.4pt + luma(220)),
  )
}

#let cover-page() = {
  let data = json("summary.json")
  let status-color = if data.overall-status == "pass" {
    rgb(25, 135, 84)
  } else if data.overall-status == "fail" {
    rgb(220, 53, 69)
  } else {
    rgb(108, 117, 125)
  }

  v(6pt)
  grid(
    columns: (1fr, auto),
    gutter: 0pt,
    stack(
      spacing: 6pt,
      text(size: 22pt, weight: "bold", tracking: 1pt)[THORNTON TOMASETTI],
      text(size: 10pt, fill: luma(90), tracking: 0.5pt)[STRUCTURAL ENGINEERING],
    ),
    align(right + top)[
      #block(fill: status-color, radius: 3pt, inset: (x: 12pt, y: 6pt))[
        #text(size: 10pt, weight: "bold", fill: white)[#upper(data.overall-status)]
      ]
    ],
  )

  v(14pt)
  line(length: 100%, stroke: 2pt + black)
  v(20pt)

  text(size: 28pt, weight: "bold")[#project.project-name]
  v(8pt)
  text(size: 14pt, fill: luma(40))[#project.subject]
  v(28pt)

  grid(
    columns: (2.2fr, 1fr),
    gutter: 30pt,

    stack(
      spacing: 8pt,
      cover-info-row("Project Number", project.project-number),
      cover-info-row("Reference", project.reference),
      cover-info-row("Code", data.code),
      cover-info-row("Revision", project.revision),
      cover-info-row("Branch", data.branch),
      cover-info-row("Version", data.version-id),
      cover-info-row("Date", project.date),
      cover-info-row("Engineer", project.engineer),
      cover-info-row("Checker", project.checker),
    ),

    stack(
      spacing: 8pt,
      text(size: parse-pt(theme.label-size), fill: luma(100), weight: "bold")[CHECKS],
      v(2pt),
      ..data.lines.map(line => {
        let dot-color = if line.status == "pass" {
          rgb(25, 135, 84)
        } else if line.status == "fail" {
          rgb(220, 53, 69)
        } else if line.status == "warn" {
          rgb(255, 193, 7)
        } else {
          luma(160)
        }
        (
          grid(
            columns: (8pt, 1fr, auto),
            gutter: 5pt,
            align: horizon,
            circle(radius: 4pt, fill: dot-color),
            text(size: 8pt)[#line.key],
            text(size: 7pt, fill: luma(100))[#upper(line.status)],
          ),
          v(3pt),
        )
      }).flatten(),
    ),
  )

  v(1fr)
  line(length: 100%, stroke: 0.5pt + luma(180))
  v(6pt)
  text(size: 7pt, fill: luma(130))[
    Generated by EtabExtension - Branch: #data.branch - Version: #data.version-id
  ]
}

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
        [*Code:* #data.code],
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
    columns: (1.45fr, 1.45fr, 1fr, 1fr, 1.45fr),
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
    columns: (auto, 1fr, auto, auto, auto),
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
    columns: (auto, 1fr, auto, auto, auto),
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
#let pier-shear-table(data) = {
  table(
    columns: (0.8fr, 0.8fr, 1.9fr, 1fr, 1fr, 1fr, 0.8fr),
    fill: (x, y) => if y == 0 { luma(220) } else { row-fill(data.annotations.at(y - 1, default: ""), y) },
    align: (x, y) => if x >= 3 { right } else { left },
    table.header(
      ..("Story", "Pier", "Combo", "Stress (psi)", "Limit", "DCR", "Status")
        .map(h => table.cell(fill: luma(220))[#h])
    ),
    ..data.rows.map(row => (
      row.story,
      row.pier,
      row.combo,
      str(calc.round(row.stress-psi, digits: 1)),
      str(calc.round(row.limit-individual, digits: 3)),
      str(calc.round(row.dcr, digits: 3)),
      if row.pass { "PASS" } else { "FAIL" },
    )).flatten(),
  )
}

#let pier-shear-page(title, data-file, chart-file) = {
  let data = json(data-file)
  let pass-str = if data.pass { "PASS" } else { "FAIL" }
  text(size: parse-pt(theme.title-size), weight: "bold")[#title]
  v(parse-pt(theme.section-gap))
  let table-body = [#align(top)[
    #text(size: parse-pt(theme.label-size), weight: "bold")[Rows: #data.rows.len() | Overall: #pass-str]
    #v(4pt)
    #pier-shear-table(data)
  ]]
  let chart-body = [#align(center)[
    #figure(
      image(chart-file, height: parse-in(theme.chart-with-table-normal-h)),
      caption: text(size: parse-pt(theme.caption-size))[Pier Shear Envelope],
    )
  ]]
  chart-table-layout(table-body, chart-body)
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

    procedures::append_definitions(&mut doc);

    // ── Generate Sequence of Section Calls ──────────────────────────────────────
    doc.push_str("\n// ── Document Sequence ────────────────────────────────\n");
    doc.push_str("#cover-page()\n\n");
    doc.push_str("#pagebreak()\n#summary-page()\n\n");

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

    procedures::append_sequence(&mut doc);

    doc
}
