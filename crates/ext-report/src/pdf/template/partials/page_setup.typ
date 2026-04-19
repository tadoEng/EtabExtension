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
