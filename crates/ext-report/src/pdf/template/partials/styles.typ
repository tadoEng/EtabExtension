#set text(font: theme.body-font, size: parse-pt(theme.body-size))
#set par(justify: false)
#set figure(numbering: "1", outlined: false)
#show figure: set block(breakable: false)
#show heading: set block(sticky: true)
#set table(stroke: 0.5pt + luma(180), inset: parse-pt(theme.table-inset))
#show table.cell.where(y: 0): set text(weight: "bold", size: parse-pt(theme.label-size))

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
