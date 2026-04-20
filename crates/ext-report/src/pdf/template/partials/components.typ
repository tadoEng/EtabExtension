#let styled-cell(fill-tag, row-idx, align-dir, content) = {
  table.cell(fill: row-fill(fill-tag, row-idx), align: align-dir)[#content]
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
  )
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
