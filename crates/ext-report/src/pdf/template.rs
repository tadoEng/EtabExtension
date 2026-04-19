use crate::pdf::procedures;
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
#set figure(numbering: "1", outlined: false)
#show figure: set block(breakable: false)
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
      set text(font: theme.body-font, size: 10pt, fill: luma(80))
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
      set text(font: theme.body-font, size: 10pt, fill: luma(80))
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

// -- Typography ladder ------------------------------------------------------
// Use these helpers for report body typography. Cover/title-block/status/caption
// helpers keep their local sizing because they have special visual roles.
#let page-title(content) = {
  text(size: parse-pt(theme.title-size), weight: "bold")[#content]
}

#let section-label(content) = {
  text(size: parse-pt(theme.label-size), weight: "bold")[#content]
}

#let body-note(content) = {
  text(size: parse-pt(theme.label-size))[#content]
}

#let ref-note(content) = {
  text(size: parse-pt(theme.label-size), fill: luma(60))[#content]
}

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

#let repeating-header(col-count, ..header-cells) = {
  table.header(
    repeat: true,
    ..header-cells,
    table.cell(
      colspan: col-count,
      fill: luma(235),
      align: right,
      inset: (x: 6pt, y: 2pt),
    )[#text(size: parse-pt(theme.label-size), fill: luma(130), style: "italic")[(continued)]],
  )
}
"##,
    );

    // ── Layout & Cell Helpers ───────────────────────────────────────────────────
    doc.push_str(
        r#"
#let styled-cell(fill-tag, row-idx, align-dir, content) = {
  table.cell(fill: row-fill(fill-tag, row-idx), align: align-dir)[#content]
}

#let status-color(status) = {
  let lowered = lower(str(status))
  if lowered == "pass" { rgb(25, 135, 84) }
  else if lowered == "fail" { rgb(220, 53, 69) }
  else if lowered == "warn" { rgb(255, 193, 7) }
  else { luma(90) }
}

#let status-text(status) = {
  text(weight: "bold", fill: status-color(status))[#upper(str(status))]
}

#let fig-caption(content) = {
  text(size: parse-pt(theme.caption-size), weight: "bold")[#content]
}

#let ext-figure(path, caption-text, height) = {
  figure(
    image(path, height: height),
    caption: fig-caption(caption-text),
  )
}

#let governing-summary(body, pass-str) = {
  v(8pt)
  block(
    inset: (x: 10pt, y: 6pt),
    fill: luma(245),
    radius: 3pt,
    width: 100%,
  )[
    #grid(
      columns: (1fr, auto),
      body,
      align(right + horizon)[#status-text(pass-str)],
    )
  ]
}

#let clamp(value, min-val, max-val) = {
  calc.max(min-val, calc.min(max-val, value))
}

#let lerp(a, b, t) = a + (b - a) * t

#let excel-three-color(value, min-val, max-val) = {
  let green = (99, 190, 123)
  let yellow = (255, 235, 132)
  let red = (248, 105, 107)
  let normalized = if max-val <= min-val {
    0.0
  } else {
    (clamp(value, min-val, max-val) - min-val) / (max-val - min-val)
  }
  if normalized <= 0.5 {
    let local = normalized * 2.0
    rgb(
      int(lerp(green.at(0), yellow.at(0), local)),
      int(lerp(green.at(1), yellow.at(1), local)),
      int(lerp(green.at(2), yellow.at(2), local)),
    )
  } else {
    let local = (normalized - 0.5) * 2.0
    rgb(
      int(lerp(yellow.at(0), red.at(0), local)),
      int(lerp(yellow.at(1), red.at(1), local)),
      int(lerp(yellow.at(2), red.at(2), local)),
    )
  }
}

#let ratio-fill(value, scale-kind) = {
  if value == none or scale-kind == none {
    none
  } else if scale-kind == "torsion_thresholds_1_2_1_4" {
    if value > 1.4 { rgb(248, 105, 107) }
    else if value > 1.2 { rgb(255, 235, 132) }
    else { rgb(99, 190, 123) }
  } else {
    let min-val = 0.0
    let max-val = if scale-kind == "shear_individual_0_10" {
      10.0
    } else if scale-kind == "shear_average_0_8" {
      8.0
    } else {
      1.0
    }
    excel-three-color(value, min-val, max-val)
  }
}

#let ratio-cell(value, scale-kind, digits: 3, align-dir: right) = {
  if value == none {
    table.cell(align: align-dir)[-]
  } else {
    table.cell(
      align: align-dir,
      fill: ratio-fill(value, scale-kind),
    )[#str(calc.round(value, digits: digits))]
  }
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
"#,
    );

    // ── Section Logic ───────────────────────────────────────────────────────────
    doc.push_str(
        r#"
#let cover-info-row(label, value) = {
  stack(
    spacing: 2pt,
    section-label[#upper(label)],
    body-note[#value],
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
      section-label[CHECKS],
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
            text(size: 7pt)[#status-text(line.status)],
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
  page-title[#project.project-name]
  body-note[#project.subject]
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
        [*Status:* #status-text(data.overall-status)],
      )
    ],
  )
  v(parse-pt(theme.section-gap))
  page-title[Report Summary]
  v(6pt)
  for line in data.lines [
    - #line.key (#status-text(line.status)) #line.message
  ]
  v(10pt)
  page-title[Checker Summary]
  v(4pt)
  table(
    columns: (1.2fr, 0.55fr, 0.9fr, 0.75fr, 0.75fr, 0.75fr, 0.65fr, 0.65fr, 1.7fr),
    fill: (x, y) => if y == 0 { luma(220) } else { row-fill("", y) },
    align: (x, y) => if x <= 3 or x == 8 { left } else { right },
    table.header(repeat: true,
      ..("Check", "Status", "Case", "Story", "Demand", "Limit", "Util.", "Margin", "Reason / Basis")
        .map(h => table.cell(fill: luma(220))[#h])
    ),
    ..data.checker-rows.map(row => (
      row.check,
      table.cell(align: left)[#status-text(row.status)],
      row.governing-case,
      row.governing-story,
      row.demand,
      row.limit,
      if row.ratio-color-value == none {
        row.utilization
      } else {
        table.cell(
          align: right,
          fill: ratio-fill(row.ratio-color-value, row.ratio-color-scale-kind),
        )[#row.utilization]
      },
      row.margin,
      row.reason,
    )).flatten(),
  )
}

#let scope-limitations-page() = {
  page-title[Scope and Limitations]
  v(parse-pt(theme.section-gap))
  enum(
    [This report is an internal screening/reporting tool generated from ETABS extracted results.],
    [Automated pass/fail status does not replace project-specific engineering judgment.],
    [Assumptions include load-case mapping, tracking-group selection, and material fallbacks where source data is incomplete.],
    [Preliminary checks (pier axial screening) are not full code-design verification.],
    [Final design and sign-off require responsible engineer review.],
  )
}

#let modal-page() = {
  let data = json("modal.json")
  page-title[Modal Participation]
  v(parse-pt(theme.section-gap))
  section-label[Mass participation threshold = #(data.threshold * 100.0)%]
  v(4pt)
  table(
    columns: (0.8fr, 0.9fr, 0.8fr, 0.8fr, 0.8fr, 0.9fr, 0.9fr, 0.9fr, 1fr),
    fill: (x, y) => if y == 0 { luma(220) } else { row-fill(data.annotations.at(y - 1, default: ""), y) },
    align: (x, y) => if x >= 1 { right } else { left },
    repeating-header(
      9,
      ..("Mode", "Period", "UX", "UY", "UZ", "Sum UX", "Sum UY", "Sum UZ", "Highlight")
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
        str(calc.round(row.uz * 100.0, digits: 1)) + "%",
        str(calc.round(row.sum-ux * 100.0, digits: 1)) + "%",
        str(calc.round(row.sum-uy * 100.0, digits: 1)) + "%",
        str(calc.round(row.sum-uz * 100.0, digits: 1)) + "%",
        highlight-label,
      )
    }).flatten(),
  )
}

#let base-reactions-table(data) = {
  table(
    columns: (1.2fr, 0.9fr, 0.8fr, 0.7fr, 0.9fr, 0.9fr, 0.9fr, 0.9fr, 0.9fr, 0.9fr),
    fill: (x, y) => if y == 0 { luma(220) } else { row-fill("", y) },
    align: (x, y) => if x >= 3 { right } else { left },
    table.header(repeat: true,
      ..("Load Case", "Type", "Step", "No.", "Fx (kip)", "Fy (kip)", "Fz (kip)", "Mx (kip-ft)", "My (kip-ft)", "Mz (kip-ft)")
        .map(h => table.cell(fill: luma(220))[#h])
    ),
    ..data.rows.map(row => (
      row.output-case,
      row.case-type,
      row.step-type,
      if row.step-number == none { "-" } else { str(calc.round(row.step-number, digits: 3)) },
      str(calc.round(row.fx-kip, digits: 2)),
      str(calc.round(row.fy-kip, digits: 2)),
      str(calc.round(row.fz-kip, digits: 2)),
      str(calc.round(row.mx-kip-ft, digits: 2)),
      str(calc.round(row.my-kip-ft, digits: 2)),
      str(calc.round(row.mz-kip-ft, digits: 2)),
    )).flatten(),
  )
}

#let base-reactions-page() = {
  let data = json("base_reactions.json")
  let pass-str = if data.pass { "PASS" } else { "FAIL" }
  page-title[Base Reaction Review]
  v(parse-pt(theme.section-gap))
  let table-body = [#align(top)[
    #section-label[Main review excludes case types Combination, LinModRitz, and Eigen.]
    #v(4pt)
    #base-reactions-table(data)
  ]]
  let chart-body = [#align(center)[
    #ext-figure("images/base_reactions.svg", [Base Reactions Envelope (kip)], parse-in(theme.chart-with-table-normal-h))
    #v(6pt)
    #align(right)[#section-label[Status: #status-text(pass-str) (X ratio #calc.round(data.ratio-x, digits: 3), Y ratio #calc.round(data.ratio-y, digits: 3))]]
  ]]
  chart-table-layout(table-body, chart-body)
}

#let story-forces-page(title, chart1, chart2) = {
  two-charts-page(title, chart1, "Shear (kip)", chart2, "Moment (kip-ft)")
}

#let drift-table(data-node) = {
  table(
    columns: data-node.groups.len() + 1,
    fill: (x, y) => if y == 0 { luma(220) } else { row-fill("", y) },
    align: (x, y) => if x == 0 { left } else { right },
    table.header(repeat: true, [Level], ..data-node.groups.map(g => [#g])),
    ..range(data-node.levels.len()).map(i => {
      let row = data-node.matrix.at(i, default: ())
      (
        data-node.levels.at(i, default: "-"),
        ..range(data-node.groups.len()).map(j => {
          let value = row.at(j, default: none)
          if value == none { "-" } else { str(calc.round(value, digits: 3)) }
        }),
      )
    }).flatten(),
  )
}

#let drift-dir-page(title, data-node, chart-file) = {
  page-title[#title]
  v(parse-pt(theme.section-gap))
  let pass-str = if data-node.pass { "PASS" } else { "FAIL" }
  let table-body = [#align(top)[
    #section-label[
      Governing: #data-node.governing-story #data-node.governing-direction #data-node.governing-case (#status-text(pass-str))
    ]
    #v(4pt)
    #drift-table(data-node)
  ]]
  let chart-body = [#stack(
    spacing: 0pt,
    align(center)[
      #ext-figure(chart-file, [Drift demand ratio by tracking group], parse-in(theme.chart-with-table-chart-h))
    ],
    v(1fr),
    align(right)[
      #stack(
        spacing: 2pt,
        body-note[Demand ratio: #calc.round(data-node.governing-demand-ratio, digits: 3)],
        body-note[Allowable ratio: #calc.round(data-node.allowable-ratio, digits: 3)],
        body-note[Utilization: #calc.round(data-node.governing-utilization * 100.0, digits: 2)% | Margin: #calc.round(data-node.governing-margin-ratio, digits: 3)],
        body-note[Basis: max drift ratio demand per group/story against allowable ratio.],
        section-label[Status: #status-text(pass-str)],
      )
    ],
  )]
  chart-table-layout(table-body, chart-body, emphasized: true)
}

#let displacement-table(data-node) = {
  table(
    columns: data-node.groups.len() + 4,
    fill: (x, y) => if y == 0 { luma(220) } else { row-fill("", y) },
    align: (x, y) => if x <= 1 { left } else { right },
    table.header(repeat: true, [Story], [Elevation (ft)], [Limit (in)], ..data-node.groups.map(g => [#g]), [Util.]),
    ..range(data-node.levels.len()).map(i => {
      let row = data-node.matrix-in.at(i, default: ())
      (
        data-node.levels.at(i, default: "-"),
        str(calc.round(data-node.level-elevations-ft.at(i, default: 0.0), digits: 2)),
        str(calc.round(data-node.level-limits-in.at(i, default: 0.0), digits: 3)),
        ..range(data-node.groups.len()).map(j => {
          let value = row.at(j, default: none)
          if value == none { "-" } else { str(calc.round(value, digits: 3)) }
        }),
        str(calc.round(data-node.level-utilization.at(i, default: 0.0) * 100.0, digits: 2)) + "%",
      )
    }).flatten(),
  )
}

#let displacement-dir-page(title, data-node, chart-file) = {
  page-title[#title]
  v(parse-pt(theme.section-gap))
  let pass-str = if data-node.pass { "PASS" } else { "FAIL" }
  let table-body = [#align(top)[
    #section-label[
      Governing: #data-node.governing-story #data-node.governing-direction #data-node.governing-case (#status-text(pass-str))
    ]
    #v(4pt)
    #displacement-table(data-node)
  ]]
  let chart-body = [#stack(
    spacing: 0pt,
    align(center)[
      #ext-figure(chart-file, [Displacement demand by tracking group], parse-in(theme.chart-with-table-chart-h))
    ],
    v(1fr),
    align(right)[
      #stack(
        spacing: 2pt,
        body-note[Demand (in): #calc.round(data-node.governing-utilization * data-node.governing-limit-in, digits: 3)],
        body-note[Limit (in): #calc.round(data-node.governing-limit-in, digits: 3) | Utilization: #calc.round(data-node.governing-utilization * 100.0, digits: 2)% | Margin: #calc.round(data-node.governing-margin * 100.0, digits: 2)%],
        body-note[Basis: per-level limit = level elevation / configured ratio divisor.],
        section-label[Status: #status-text(pass-str)],
      )
    ],
  )]
  chart-table-layout(table-body, chart-body, emphasized: true)
}

#let torsional-dir-page(title, data-node, chart-file) = {
  page-title[#title]
  v(parse-pt(theme.section-gap))
  let pass-str = if data-node.has-type-b { "FAIL" } else { "PASS" }
  if data-node.has-rows {
    section-label[
      Governing story: #data-node.governing-story | Case: #data-node.governing-case | Pair: #data-node.governing-joint-a / #data-node.governing-joint-b | Step: #data-node.governing-step
    ]
    body-note[
      Drift A: #calc.round(data-node.governing-drift-a, digits: 3) | Drift B: #calc.round(data-node.governing-drift-b, digits: 3)
    ]
    body-note[DeltaMax: #calc.round(data-node.governing-delta-max, digits: 3) | DeltaAvg: #calc.round(data-node.governing-delta-avg, digits: 3)]
    box(
      inset: (x: 4pt, y: 2pt),
      radius: 2pt,
      fill: ratio-fill(data-node.governing-ratio-color-value, data-node.governing-ratio-color-scale-kind),
    )[
      #section-label[Ratio: #calc.round(data-node.governing-ratio, digits: 3)]
    ]
    body-note[
      Thresholds: Type A > #calc.round(data-node.type-a-threshold, digits: 2), Type B > #calc.round(data-node.type-b-threshold, digits: 2) | Classification: #data-node.classification
    ]
  } else {
    section-label[#data-node.no-data-note]
    body-note[Thresholds: Type A > #calc.round(data-node.type-a-threshold, digits: 2), Type B > #calc.round(data-node.type-b-threshold, digits: 2) | Classification: #data-node.classification]
  }
  v(4pt)
  let torsion-rows = if data-node.has-rows {
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
    (
      data-node.no-data-note, "", "", "", "", "", "", "", "", "",
    )
  }
  table(
    columns: (1fr, 1fr, 1fr, 1fr, 0.65fr, 0.8fr, 0.8fr, 0.9fr, 0.9fr, 0.8fr),
    fill: (x, y) => if y == 0 { luma(220) } else { row-fill(data-node.annotations.at(y - 1, default: ""), y) },
    align: (x, y) => if x >= 4 { right } else { left },
    table.header(repeat: true,
      ..("Story", "Case", "Joint A", "Joint B", "Step", "Drift A", "Drift B", "DeltaMax", "DeltaAvg", "Ratio")
        .map(h => table.cell(fill: luma(220))[#h])
    ),
    ..torsion-rows,
  )
  v(6pt)
  align(center)[
    #ext-figure(chart-file, [Story governing torsional ratio with 1.2/1.4 thresholds], parse-in(theme.chart-with-table-chart-h))
  ]
  v(4pt)
  align(right)[#section-label[Status: #status-text(pass-str)]]
}
"#,
    );

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

    procedures::append_definitions(&mut doc);

    // ── Generate Sequence of Section Calls ──────────────────────────────────────
    doc.push_str("\n// ── Document Sequence ────────────────────────────────\n");
    doc.push_str("#cover-page()\n\n");
    doc.push_str("#pagebreak()\n#summary-page()\n\n");
    doc.push_str("#pagebreak()\n#scope-limitations-page()\n\n");

    if calc.modal.is_some() {
        doc.push_str("#pagebreak()\n#modal-page()\n\n");
    }

    if calc.base_reactions.is_some() {
        doc.push_str("#pagebreak()\n#base-reactions-page()\n\n");
    }

    if calc.story_forces.is_some() {
        doc.push_str("#pagebreak()\n#story-force-review-page([Story Forces — X Direction], \"images/story_force_vx.svg\", [Story Shear Vx (kip)], \"images/story_force_my.svg\", [Story Moment My (kip·ft)])\n\n");
        doc.push_str("#pagebreak()\n#story-force-review-page([Story Forces — Y Direction], \"images/story_force_vy.svg\", [Story Shear Vy (kip)], \"images/story_force_mx.svg\", [Story Moment Mx (kip·ft)])\n\n");
    }

    if calc.drift_wind.is_some() {
        doc.push_str("#pagebreak()\n#let dw = json(\"drift_wind.json\")\n#drift-review-pair-page([Wind Drift Review], dw, \"images/drift_wind_x.svg\", [Wind Drift Ratio — X Direction], \"images/drift_wind_y.svg\", [Wind Drift Ratio — Y Direction])\n\n");
    }

    if calc.drift_seismic.is_some() {
        doc.push_str("#pagebreak()\n#let ds = json(\"drift_seismic.json\")\n#drift-review-pair-page([Seismic Drift Review], ds, \"images/drift_seismic_x.svg\", [Seismic Drift Ratio — X Direction], \"images/drift_seismic_y.svg\", [Seismic Drift Ratio — Y Direction])\n\n");
    }

    if calc.displacement_wind.is_some() {
        doc.push_str("#pagebreak()\n#let dpw = json(\"displacement_wind.json\")\n#displacement-review-pair-page([Wind Displacement Review], dpw, \"images/displacement_wind_x.svg\", [Wind Displacement — X Direction (in)], \"images/displacement_wind_y.svg\", [Wind Displacement — Y Direction (in)])\n\n");
    }

    if calc.torsional.is_some() {
        doc.push_str("#let tor = json(\"torsional.json\")\n");
        doc.push_str("#pagebreak()\n#torsion-review-pair-page([Torsional Irregularity Review], tor, \"images/torsional_x.svg\", \"images/torsional_y.svg\")\n\n");
        doc.push_str("#pagebreak()\n#torsion-verify-pair-page([Torsional Irregularity Verification], tor)\n\n");
    }

    if calc.pier_shear_stress_wind.is_some() {
        doc.push_str("#pagebreak()\n#let psw = json(\"pier_shear_wind.json\")\n#pier-shear-review-pair-page([Pier Shear Wind Review], psw, \"images/pier_shear_stress_wind_x.svg\", [Pier Shear Stress Ratio Wind — X Walls], \"images/pier_shear_stress_wind_y.svg\", [Pier Shear Stress Ratio Wind — Y Walls])\n\n");
        doc.push_str(
            "#pagebreak()\n#pier-shear-verify-pair-page([Pier Shear Wind Verification], psw)\n\n",
        );
        doc.push_str("#pagebreak()\n#pier-shear-average-review-page([Pier Shear Wind Average Review], \"pier_shear_wind.json\", \"images/pier_shear_stress_wind_avg.svg\")\n\n");
        doc.push_str("#pagebreak()\n#pier-shear-average-verify-page([Pier Shear Wind Average Verification], \"pier_shear_wind.json\")\n\n");
    }

    if calc.pier_shear_stress_seismic.is_some() {
        doc.push_str("#pagebreak()\n#let pss = json(\"pier_shear_seismic.json\")\n#pier-shear-review-pair-page([Pier Shear Seismic Review], pss, \"images/pier_shear_stress_seismic_x.svg\", [Pier Shear Stress Ratio Seismic — X Walls], \"images/pier_shear_stress_seismic_y.svg\", [Pier Shear Stress Ratio Seismic — Y Walls])\n\n");
        doc.push_str("#pagebreak()\n#pier-shear-verify-pair-page([Pier Shear Seismic Verification], pss)\n\n");
        doc.push_str("#pagebreak()\n#pier-shear-average-review-page([Pier Shear Seismic Average Review], \"pier_shear_seismic.json\", \"images/pier_shear_stress_seismic_avg.svg\")\n\n");
        doc.push_str("#pagebreak()\n#pier-shear-average-verify-page([Pier Shear Seismic Average Verification], \"pier_shear_seismic.json\")\n\n");
    }

    if calc.pier_axial_stress.is_some() {
        doc.push_str("#pagebreak()\n#single-chart-page([Pier Axial - Gravity], \"images/pier_axial_gravity.svg\", [Pier Axial Stress — Gravity (ksi)])\n\n");
        doc.push_str("#pagebreak()\n#single-chart-page([Pier Axial - Wind], \"images/pier_axial_wind.svg\", [Pier Axial Stress — Wind (ksi)])\n\n");
        doc.push_str("#pagebreak()\n#single-chart-page([Pier Axial - Seismic], \"images/pier_axial_seismic.svg\", [Pier Axial Stress — Seismic (ksi)])\n\n");
    }

    procedures::append_sequence(&mut doc);

    doc
}

#[cfg(test)]
mod tests {
    use super::build_typst_document;
    use ext_calc::CalcRunner;
    use ext_calc::code_params::CodeParams;
    use ext_calc::output::CalcOutput;
    use ext_db::config::Config;
    use std::path::PathBuf;

    fn fixture_calc_output() -> CalcOutput {
        let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../ext-calc/tests/fixtures/results_realistic");
        let path = fixture_dir.join("calc_output.json");
        if path.exists() {
            let text = std::fs::read_to_string(path).expect("fixture json should be readable");
            serde_json::from_str(&text).expect("fixture json should deserialize")
        } else {
            let config = Config::load(&fixture_dir).expect("fixture config should load");
            let params = CodeParams::from_config(&config).expect("code params should build");
            CalcRunner::run_all(
                fixture_dir.as_path(),
                fixture_dir.as_path(),
                &params,
                "fixture",
                "main",
            )
            .expect("fixture calc output should build")
        }
    }

    fn typst_block<'a>(typst: &'a str, start: &str, end: &str) -> &'a str {
        let start_idx = typst
            .find(start)
            .unwrap_or_else(|| panic!("missing Typst block start: {start}"));
        let rest = &typst[start_idx..];
        let end_idx = rest
            .find(end)
            .unwrap_or_else(|| panic!("missing Typst block end after {start}: {end}"));
        &rest[..end_idx]
    }

    #[test]
    fn directional_sections_render_review_page_before_verification_page() {
        let calc = fixture_calc_output();
        let typst = build_typst_document(&calc);

        for (review, verify) in [
            (
                "#torsion-review-pair-page([Torsional Irregularity Review]",
                "#torsion-verify-pair-page([Torsional Irregularity Verification]",
            ),
            (
                "#pier-shear-review-pair-page([Pier Shear Wind Review]",
                "#pier-shear-verify-pair-page([Pier Shear Wind Verification]",
            ),
            (
                "#pier-shear-review-pair-page([Pier Shear Seismic Review]",
                "#pier-shear-verify-pair-page([Pier Shear Seismic Verification]",
            ),
            (
                "#pier-shear-average-review-page([Pier Shear Wind Average Review]",
                "#pier-shear-average-verify-page([Pier Shear Wind Average Verification]",
            ),
            (
                "#pier-shear-average-review-page([Pier Shear Seismic Average Review]",
                "#pier-shear-average-verify-page([Pier Shear Seismic Average Verification]",
            ),
        ] {
            let review_idx = typst
                .find(review)
                .unwrap_or_else(|| panic!("missing review marker: {review}"));
            let verify_idx = typst
                .find(verify)
                .unwrap_or_else(|| panic!("missing verification marker: {verify}"));
            assert!(
                review_idx < verify_idx,
                "review marker should appear before verification marker for '{review}'"
            );
        }
    }

    #[test]
    fn template_uses_directional_pier_shear_assets_and_removes_axial_screening_page() {
        let calc = fixture_calc_output();
        let typst = build_typst_document(&calc);

        for image in [
            "images/pier_shear_stress_wind_x.svg",
            "images/pier_shear_stress_wind_y.svg",
            "images/pier_shear_stress_seismic_x.svg",
            "images/pier_shear_stress_seismic_y.svg",
        ] {
            assert!(
                typst.contains(image),
                "missing directional image asset {image}"
            );
        }

        assert!(
            !typst.contains("#pier-axial-assumptions(\"pier_axial_stress.json\")"),
            "axial preliminary-screening page call should be removed from sequence"
        );

        assert!(
            typst.contains("#show figure: set block(breakable: false)"),
            "figures should be non-breakable to avoid orphaned chart/table fragments"
        );
        assert!(
            typst.contains("#set figure(numbering: \"1\", outlined: false)"),
            "figures should be numbered globally"
        );
        assert!(
            !typst.contains("numbering: none"),
            "old unnumbered figure setting should be removed"
        );
        assert!(
            typst.contains("#let ext-figure(path, caption-text, height)"),
            "chart images should route through the shared ext-figure helper"
        );
        assert!(
            typst.contains("#let two-chart-row(chart1, cap1, chart2, cap2)"),
            "side-by-side chart review pages should share two-chart-row"
        );
        assert!(
            typst.contains("governing-summary("),
            "review pages should use structured governing-summary callouts"
        );
        assert!(
            typst
                .matches("text(size: parse-pt(theme.title-size)")
                .count()
                == 1,
            "page-title should be the only title-size text definition"
        );
        assert!(
            typst
                .matches("text(size: parse-pt(theme.label-size)")
                .count()
                == 6,
            "label-size text calls should be limited to helper definitions and worked-example result helpers"
        );
        assert!(
            !typst.contains("text(size: parse-pt(theme.body-size)"),
            "body-note/section-label helpers should replace all inline body-size text calls"
        );
        assert!(
            !typst.contains("text(size: title-size") && !typst.contains("text(size: label-size"),
            "typography migration should not use local size aliases"
        );
        for helper_usage in ["page-title[", "section-label[", "body-note[", "ref-note["] {
            assert!(
                typst.contains(helper_usage),
                "expected direct typography helper usage: {helper_usage}"
            );
        }
        assert!(
            typst.contains("block(breakable: false)[#stack(spacing: 4pt, section-label[X Direction], drift-table(data.x))]"),
            "drift verification columns should be unbreakable and use typography helpers"
        );
        assert!(
            typst.contains("block(breakable: false)[#stack(spacing: 4pt, section-label[X Direction], displacement-table(data.x))]"),
            "displacement verification columns should be unbreakable and use typography helpers"
        );
        assert!(
            typst.matches("with-divider(").count() >= 5,
            "with-divider helper should be used in verification tables and worked examples"
        );
        for (start, end) in [
            (
                "#let two-chart-row(chart1, cap1, chart2, cap2)",
                "#let with-divider(left-content, right-content)",
            ),
            (
                "#let story-force-review-page(title, chart1, cap1, chart2, cap2)",
                "#let drift-review-pair-page",
            ),
            ("#let drift-review-pair-page", "#let drift-verify-pair-page"),
            (
                "#let displacement-review-pair-page",
                "#let displacement-verify-pair-page",
            ),
            (
                "#let torsion-review-pair-page",
                "#let torsion-verify-pair-page",
            ),
            (
                "#let pier-shear-review-pair-page",
                "#let pier-shear-verify-pair-page",
            ),
        ] {
            assert!(
                !typst_block(&typst, start, end).contains("with-divider("),
                "chart review helper should not use with-divider: {start}"
            );
        }
        assert!(
            !typst.contains("#drift-verify-pair-page([Wind Drift Verification]")
                && !typst.contains("#drift-verify-pair-page([Seismic Drift Verification]")
                && !typst
                    .contains("#displacement-verify-pair-page([Wind Displacement Verification]")
                && !typst.contains("#drift-verify-pair-page(")
                && !typst.contains("#displacement-verify-pair-page("),
            "duplicative drift/displacement verification page calls should be removed"
        );
        let table_header_count = typst.matches("table.header(").count();
        let inline_repeat_header_count = typst.matches("table.header(repeat: true,").count();
        assert_eq!(
            table_header_count,
            inline_repeat_header_count + 1,
            "every ordinary table.header should use repeat: true; only repeating-header may call table.header with multiline args"
        );
        assert!(
            typst.contains("table.header(\n    repeat: true,")
                && !typst.contains("table.header(repeat: false"),
            "repeating-header should call table.header(repeat: true) and no header should disable repeats"
        );
        assert!(
            typst.matches("repeating-header(").count() >= 4,
            "repeating-header should be defined and used for modal, torsion, and pier-shear directional tables"
        );
        assert!(
            typst.contains(
                "block(breakable: false)[\n      #align(center)[\n        #ext-figure(chart-file"
            ),
            "pier-shear-average-review-page chart and summary should be wrapped in a non-breakable block"
        );
        assert!(
            typst.contains("parse-pt(theme.label-size) + 2pt"),
            "worked-example boxed results should use direct label-size + 2pt sizing"
        );
        assert!(
            typst.contains("enum("),
            "scope-limitations page should use a Typst enum list"
        );
        assert!(
            !typst.contains("align(center)[image(chart1"),
            "story-force chart1 must not contain unevaluated image() call"
        );
        assert!(
            !typst.contains("align(center)[image(chart2"),
            "story-force chart2 must not contain unevaluated image() call"
        );
        assert!(
            !typst.contains("align(center)[\n      figure(\n        image(chart-file"),
            "average shear review must not contain unevaluated figure() call"
        );
        assert!(
            !typst.contains("align(center)[text(size: parse-pt(theme.caption-size)"),
            "story-force captions should not leak as raw caption text inside stacks"
        );
        assert!(
            !typst.contains("image(\"images/"),
            "literal chart image paths should be wrapped by ext-figure"
        );
        for raw_dynamic_image in [
            "image(chart1",
            "image(chart2",
            "image(chart-x",
            "image(chart-y",
            "image(chart-file",
        ] {
            assert!(
                !typst.contains(raw_dynamic_image),
                "dynamic chart image call should be wrapped by ext-figure: {raw_dynamic_image}"
            );
        }
        for caption in [
            "Story Shear Vx (kip)",
            "Story Moment My (kip·ft)",
            "Wind Drift Ratio — X Direction",
            "Seismic Drift Ratio — Y Direction",
            "Wind Displacement — X Direction (in)",
            "Torsional Ratio — X Direction",
            "Pier Shear Stress Ratio Wind — X Walls",
            "Pier Axial Stress — Seismic (ksi)",
        ] {
            assert!(
                typst.contains(caption),
                "missing expected chart caption: {caption}"
            );
        }
        assert!(
            typst.contains("#single-chart-page([Pier Axial - Gravity]")
                && typst.contains("#single-chart-page([Pier Axial - Wind]")
                && typst.contains("#single-chart-page([Pier Axial - Seismic]"),
            "pier axial should remain three separate pages"
        );
        for procedure_marker in [
            "#let given-table(pairs)",
            "#let calc-step(n, formula, substitution, result)",
            "#let calc-result(label, value, pass)",
            "if is-executive",
        ] {
            assert!(
                typst.contains(procedure_marker),
                "missing worked-example marker: {procedure_marker}"
            );
        }
        assert!(
            !typst.contains("text(size: 9pt"),
            "worked examples should not hardcode text below theme.label-size"
        );
    }
}
