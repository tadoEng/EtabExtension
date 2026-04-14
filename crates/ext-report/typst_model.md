
    Docs
    Reference
    Model
    Table

table
Element

A table of items.

Tables are used to arrange content in cells. Cells can contain arbitrary content, including multiple paragraphs and are specified in row-major order. For a hands-on explanation of all the ways you can use and customize tables in Typst, check out the Table Guide.

Because tables are just grids with different defaults for some cell properties (notably stroke and inset), refer to the grid documentation for more information on how to size the table tracks and specify the cell appearance properties.

If you are unsure whether you should be using a table or a grid, consider whether the content you are arranging semantically belongs together as a set of related data points or similar or whether you are just want to enhance your presentation by arranging unrelated content in a grid. In the former case, a table is the right choice, while in the latter case, a grid is more appropriate. Furthermore, Assistive Technology (AT) like screen readers will announce content in a table as tabular while a grid's content will be announced no different than multiple content blocks in the document flow. AT users will be able to navigate tables two-dimensionally by cell.

Note that, to override a particular cell's properties or apply show rules on table cells, you can use the table.cell element. See its documentation for more information.

Although the table and the grid share most properties, set and show rules on one of them do not affect the other. Locating most of your styling in set and show rules is recommended, as it keeps the table's actual usages clean and easy to read. It also allows you to easily change the appearance of all tables in one place.

To give a table a caption and make it referenceable, put it into a figure.
Example

The example below demonstrates some of the most common table options.

#table(
  columns: (1fr, auto, auto),
  inset: 10pt,
  align: horizon,
  table.header(
    [], [*Volume*], [*Parameters*],
  ),
  image("cylinder.svg"),
  $ pi h (D^2 - d^2) / 4 $,
  [
    $h$: height \
    $D$: outer radius \
    $d$: inner radius
  ],
  image("tetrahedron.svg"),
  $ sqrt(2) / 12 a^3 $,
  [$a$: edge length]
)

Preview

Much like with grids, you can use table.cell to customize the appearance and the position of each cell.

#set table(
  stroke: none,
  gutter: 0.2em,
  fill: (x, y) =>
    if x == 0 or y == 0 { gray },
  inset: (right: 1.5em),
)

#show table.cell: it => {
  if it.x == 0 or it.y == 0 {
    set text(white)
    strong(it)
  } else if it.body == [] {
    // Replace empty cells with 'N/A'
    pad(..it.inset)[_N/A_]
  } else {
    it
  }
}

#let a = table.cell(
  fill: green.lighten(60%),
)[A]
#let b = table.cell(
  fill: aqua.lighten(60%),
)[B]

#table(
  columns: 4,
  [], [Exam 1], [Exam 2], [Exam 3],

  [John], [], a, [],
  [Mary], [], a, a,
  [Robert], b, a, b,
)

Preview
Accessibility

Tables are challenging to consume for users of Assistive Technology (AT). To make the life of AT users easier, we strongly recommend that you use table.header and table.footer to mark the header and footer sections of your table. This will allow AT to announce the column labels for each cell.

Because navigating a table by cell is more cumbersome than reading it visually, you should consider making the core information in your table available as text as well. You can do this by wrapping your table in a figure and using its caption to summarize the table's content.
Parameters
table(
columns: autointrelativefractionarray,rows: autointrelativefractionarray,gutter: autointrelativefractionarray,column-gutter: autointrelativefractionarray,row-gutter: autointrelativefractionarray,inset: relativearraydictionaryfunction,align: autoarrayalignmentfunction,fill: nonecolorgradientarraytilingfunction,stroke: nonelengthcolorgradientarraystroketilingdictionaryfunction,..content,
) -> content
columns
auto or int or relative or fraction or array
Settable

The column sizes. See the grid documentation for more information on track sizing.

Default: ()
rows
auto or int or relative or fraction or array
Settable

The row sizes. See the grid documentation for more information on track sizing.

Default: ()
gutter
auto or int or relative or fraction or array

The gaps between rows and columns. This is a shorthand for setting column-gutter and row-gutter to the same value. See the grid documentation for more information on gutters.

Default: ()
column-gutter
auto or int or relative or fraction or array
Settable

The gaps between columns. Takes precedence over gutter. See the grid documentation for more information on gutters.

Default: ()
row-gutter
auto or int or relative or fraction or array
Settable

The gaps between rows. Takes precedence over gutter. See the grid documentation for more information on gutters.

Default: ()
inset
relative or array or dictionary or function
Settable

How much to pad the cells' content.

To specify the same inset for all cells, use a single length for all sides, or a dictionary of lengths for individual sides. See the box's documentation for more details.

To specify a varying inset for different cells, you can:

    use a single, uniform inset for all cells
    use an array of insets for each column
    use a function that maps a cell's X/Y position (both starting from zero) to its inset

See the grid documentation for more details.

Preview

Default: 0% + 5pt
align
auto or array or alignment or function
Settable

How to align the cells' content.

If set to auto, the outer alignment is used.

You can specify the alignment in any of the following fashions:

    use a single alignment for all cells
    use an array of alignments corresponding to each column
    use a function that maps a cell's X/Y position (both starting from zero) to its alignment

See the Table Guide for details.

Preview

Default: auto
fill
none or color or gradient or array or tiling or function
Settable

How to fill the cells.

This can be:

    a single fill for all cells
    an array of fill corresponding to each column
    a function that maps a cell's position to its fill

Most notably, arrays and functions are useful for creating striped tables. See the Table Guide for more details.

Preview

Default: none
stroke
none or length or color or gradient or array or stroke or tiling or dictionary or function
Settable

How to stroke the cells.

Strokes can be disabled by setting this to none.

If it is necessary to place lines which can cross spacing between cells produced by the gutter option, or to override the stroke between multiple specific cells, consider specifying one or more of table.hline and table.vline alongside your table cells.

To specify the same stroke for all cells, use a single stroke for all sides, or a dictionary of strokes for individual sides. See the rectangle's documentation for more details.

To specify varying strokes for different cells, you can:

    use a single stroke for all cells
    use an array of strokes corresponding to each column
    use a function that maps a cell's position to its stroke

See the Table Guide for more details.

Default: 1pt + black
children
content
Required
Positional
Variadic

The contents of the table cells, plus any extra table lines specified with the table.hline and table.vline elements.
Definitions
cell
Element

A cell in the table. Use this to position a cell manually or to apply styling. To do the latter, you can either use the function to override the properties for a particular cell, or use it in show rules to apply certain styles to multiple cells at once.

Perhaps the most important use case of table.cell is to make a cell span multiple columns and/or rows with the colspan and rowspan fields.

#show table.cell.where(y: 0): strong
#set table(
  stroke: (x, y) => if y == 0 {
    (bottom: 0.7pt + black)
  },
  align: (x, y) => (
    if x > 0 { center }
    else { left }
  )
)

#table(
  columns: 3,
  table.header(
    [Substance],
    [Subcritical °C],
    [Supercritical °C],
  ),
  [Hydrochloric Acid],
  [12.0], [92.1],
  [Sodium Myreth Sulfate],
  [16.6], [104],
  [Potassium Hydroxide],
  table.cell(colspan: 2)[24.7],
)

Preview

For example, you can override the fill, alignment or inset for a single cell:

Preview

You may also apply a show rule on table.cell to style all cells at once. Combined with selectors, this allows you to apply styles based on a cell's position:

Preview
table.cell(
content,x: autoint,y: autoint,colspan: int,rowspan: int,inset: autorelativedictionary,align: autoalignment,fill: noneautocolorgradienttiling,stroke: nonelengthcolorgradientstroketilingdictionary,breakable: autobool,
) -> content
body
content
Required
Positional

The cell's body.
x
auto or int
Settable

The cell's column (zero-indexed). Functions identically to the x field in grid.cell.

Default: auto
y
auto or int
Settable

The cell's row (zero-indexed). Functions identically to the y field in grid.cell.

Default: auto
colspan
int
Settable

The amount of columns spanned by this cell.

Default: 1
rowspan
int
Settable

The amount of rows spanned by this cell.

Default: 1
inset
auto or relative or dictionary
Settable

The cell's inset override.

Default: auto
align
auto or alignment
Settable

The cell's alignment override.

Default: auto
fill
none or auto or color or gradient or tiling
Settable

The cell's fill override.

Default: auto
stroke
none or length or color or gradient or stroke or tiling or dictionary
Settable

The cell's stroke override.

Default: (:)
breakable
auto or bool
Settable

Whether rows spanned by this cell can be placed in different pages. When equal to auto, a cell spanning only fixed-size rows is unbreakable, while a cell spanning at least one auto-sized row is breakable.

Default: auto
hline
Element

A horizontal line in the table.

Overrides any per-cell stroke, including stroke specified through the table's stroke field. Can cross spacing between cells created through the table's column-gutter option.

Use this function instead of the table's stroke field if you want to manually place a horizontal line at a specific position in a single table. Consider using table's stroke field or table.cell's stroke field instead if the line you want to place is part of all your tables' designs.

#set table.hline(stroke: .6pt)

#table(
  stroke: none,
  columns: (auto, 1fr),
  [09:00], [Badge pick up],
  [09:45], [Opening Keynote],
  [10:30], [Talk: Typst's Future],
  [11:15], [Session: Good PRs],
  table.hline(start: 1),
  [Noon], [_Lunch break_],
  table.hline(start: 1),
  [14:00], [Talk: Tracked Layout],
  [15:00], [Talk: Automations],
  [16:00], [Workshop: Tables],
  table.hline(),
  [19:00], [Day 1 Attendee Mixer],
)

Preview
table.hline(
y: autoint,start: int,end: noneint,stroke: nonelengthcolorgradientstroketilingdictionary,position: alignment,
) -> content
y
auto or int
Settable

The row above which the horizontal line is placed (zero-indexed). Functions identically to the y field in grid.hline.

Default: auto
start
int
Settable

The column at which the horizontal line starts (zero-indexed, inclusive).

Default: 0
end
none or int
Settable

The column before which the horizontal line ends (zero-indexed, exclusive).

Default: none
stroke
none or length or color or gradient or stroke or tiling or dictionary
Settable

The line's stroke.

Specifying none removes any lines previously placed across this line's range, including hlines or per-cell stroke below it.

Default: 1pt + black
position
alignment
Settable

The position at which the line is placed, given its row (y) - either top to draw above it or bottom to draw below it.

This setting is only relevant when row gutter is enabled (and shouldn't be used otherwise - prefer just increasing the y field by one instead), since then the position below a row becomes different from the position above the next row due to the spacing between both.

Default: top
vline
Element

A vertical line in the table. See the docs for grid.vline for more information regarding how to use this element's fields.

Overrides any per-cell stroke, including stroke specified through the table's stroke field. Can cross spacing between cells created through the table's row-gutter option.

Similar to table.hline, use this function if you want to manually place a vertical line at a specific position in a single table and use the table's stroke field or table.cell's stroke field instead if the line you want to place is part of all your tables' designs.
table.vline(
x: autoint,start: int,end: noneint,stroke: nonelengthcolorgradientstroketilingdictionary,position: alignment,
) -> content
x
auto or int
Settable

The column before which the vertical line is placed (zero-indexed). Functions identically to the x field in grid.vline.

Default: auto
start
int
Settable

The row at which the vertical line starts (zero-indexed, inclusive).

Default: 0
end
none or int
Settable

The row on top of which the vertical line ends (zero-indexed, exclusive).

Default: none
stroke
none or length or color or gradient or stroke or tiling or dictionary
Settable

The line's stroke.

Specifying none removes any lines previously placed across this line's range, including vlines or per-cell stroke below it.

Default: 1pt + black
position
alignment
Settable

The position at which the line is placed, given its column (x) - either start to draw before it or end to draw after it.

The values left and right are also accepted, but discouraged as they cause your table to be inconsistent between left-to-right and right-to-left documents.

This setting is only relevant when column gutter is enabled (and shouldn't be used otherwise - prefer just increasing the x field by one instead), since then the position after a column becomes different from the position before the next column due to the spacing between both.

Default: start
header
Element

A repeatable table header.

You should wrap your tables' heading rows in this function even if you do not plan to wrap your table across pages because Typst uses this function to attach accessibility metadata to tables and ensure Universal Access to your document.

You can use the repeat parameter to control whether your table's header will be repeated across pages.

Currently, this function is unsuitable for creating a header column or single header cells. Either use regular cells, or, if you are exporting a PDF, you can also use the pdf.header-cell function to mark a cell as a header cell. Likewise, you can use pdf.data-cell to mark cells in this function as data cells. Note that these functions are not final and thus only available when you enable the a11y-extras feature (see the PDF module documentation for details).

#set page(height: 11.5em)
#set table(
  fill: (x, y) =>
    if x == 0 or y == 0 {
      gray.lighten(40%)
    },
  align: right,
)

#show table.cell.where(x: 0): strong
#show table.cell.where(y: 0): strong

#table(
  columns: 4,
  table.header(
    [], [Blue chip],
    [Fresh IPO], [Penny st'k],
  ),
  table.cell(
    rowspan: 6,
    align: horizon,
    rotate(-90deg, reflow: true)[
      *USD / day*
    ],
  ),
  [0.20], [104], [5],
  [3.17], [108], [4],
  [1.59], [84],  [1],
  [0.26], [98],  [15],
  [0.01], [195], [4],
  [7.34], [57],  [2],
)

Preview Preview
table.header(
repeat: bool,level: int,..content,
) -> content
repeat
bool
Settable

Whether this header should be repeated across pages.

Default: true
level
int
Settable

The level of the header. Must not be zero.

This allows repeating multiple headers at once. Headers with different levels can repeat together, as long as they have ascending levels.

Notably, when a header with a lower level starts repeating, all higher or equal level headers stop repeating (they are "replaced" by the new header).

Default: 1
children
content
Required
Positional
Variadic

The cells and lines within the header.
footer
Element

A repeatable table footer.

Just like the table.header element, the footer can repeat itself on every page of the table. This is useful for improving legibility by adding the column labels in both the header and footer of a large table, totals, or other information that should be visible on every page.

No other table cells may be placed after the footer.
table.footer(
repeat: bool,..content,
) -> content
repeat
bool
Settable

Whether this footer should be repeated across pages.

Default: true
children
content
Required
Positional
Variadic

The cells and lines within the footer.


    Docs
    Reference
    Model
    Figure

figure
Element

A figure with an optional caption.

Automatically detects its kind to select the correct counting track. For example, figures containing images will be numbered separately from figures containing tables.
Examples

The example below shows a basic figure with an image:

@glacier shows a glacier. Glaciers
are complex systems.

#figure(
  image("glacier.jpg", width: 80%),
  caption: [A curious figure.],
) <glacier>

Preview

You can also insert tables into figures to give them a caption. The figure will detect this and automatically use a separate counter.

#figure(
  table(
    columns: 4,
    [t], [1], [2], [3],
    [y], [0.3s], [0.4s], [0.8s],
  ),
  caption: [Timing results],
)

Preview

This behaviour can be overridden by explicitly specifying the figure's kind. All figures of the same kind share a common counter.
Figure behaviour

By default, figures are placed within the flow of content. To make them float to the top or bottom of the page, you can use the placement argument.

If your figure is too large and its contents are breakable across pages (e.g. if it contains a large table), then you can make the figure itself breakable across pages as well with this show rule:

#show figure: set block(breakable: true)

See the block documentation for more information about breakable and non-breakable blocks.
Caption customization

You can modify the appearance of the figure's caption with its associated caption function. In the example below, we emphasize all captions:

#show figure.caption: emph

#figure(
  rect[Hello],
  caption: [I am emphasized!],
)

Preview

By using a where selector, we can scope such rules to specific kinds of figures. For example, to position the caption above tables, but keep it below for all other kinds of figures, we could write the following show-set rule:

#show figure.where(
  kind: table
): set figure.caption(position: top)

#figure(
  table(columns: 2)[A][B][C][D],
  caption: [I'm up here],
)

Preview
Accessibility

You can use the alt parameter to provide an alternative description of the figure for screen readers and other Assistive Technology (AT). Refer to its documentation to learn more.

You can use figures to add alternative descriptions to paths, shapes, or visualizations that do not have their own alt parameter. If your graphic is purely decorative and does not have a semantic meaning, consider wrapping it in pdf.artifact instead, which will hide it from AT when exporting to PDF.

AT will always read the figure at the point where it appears in the document, regardless of its placement. Put its markup where it would make the most sense in the reading order.
Parameters
figure(
content,alt: nonestr,placement: noneautoalignment,scope: str,caption: nonecontent,kind: autostrfunction,supplement: noneautocontentfunction,numbering: nonestrfunction,gap: length,outlined: bool,
) -> content
body
content
Required
Positional

The content of the figure. Often, an image.
alt
none or str
Settable

An alternative description of the figure.

When you add an alternative description, AT will read both it and the caption (if any). However, the content of the figure itself will be skipped.

When the body of your figure is an image with its own alt text set, this parameter should not be used on the figure element. Likewise, do not use this parameter when the figure contains a table, code, or other content that is already accessible. In such cases, the content of the figure will be read by AT, and adding an alternative description would lead to a loss of information.

You can learn how to write good alternative descriptions in the Accessibility Guide.

Default: none
placement
none or auto or alignment
Settable

The figure's placement on the page.

    none: The figure stays in-flow exactly where it was specified like other content.
    auto: The figure picks top or bottom depending on which is closer.
    top: The figure floats to the top of the page.
    bottom: The figure floats to the bottom of the page.

The gap between the main flow content and the floating figure is controlled by the clearance argument on the place function.

Preview Preview

Default: none
scope
str
Settable

Relative to which containing scope the figure is placed.

Set this to "parent" to create a full-width figure in a two-column document.

Has no effect if placement is none.

Preview Preview
Variant	Details
"column"	

Place into the current column.
"parent"	

Place relative to the parent, letting the content span over all columns.

Default: "column"
caption
none or content
Settable

The figure's caption.

Default: none
kind
auto or str or function
Settable

The kind of figure this is.

All figures of the same kind share a common counter.

If set to auto, the figure will try to automatically determine its kind based on the type of its body. Automatically detected kinds are tables and code. In other cases, the inferred kind is that of an image.

Setting this to something other than auto will override the automatic detection. This can be useful if

    you wish to create a custom figure type that is not an image, a table or code,
    you want to force the figure to use a specific counter regardless of its content.

You can set the kind to be an element function or a string. If you set it to an element function other than table, raw, or image, you will need to manually specify the figure's supplement.

Preview

If you want to modify a counter to skip a number or reset the counter, you can access the counter of each kind of figure with a where selector:

    For tables: counter(figure.where(kind: table))
    For images: counter(figure.where(kind: image))
    For a custom kind: counter(figure.where(kind: kind))

Preview

To conveniently use the correct counter in a show rule, you can access the counter field. There is an example of this in the documentation of the figure.caption element's body field.

Default: auto
supplement
none or auto or content or function
Settable

The figure's supplement.

If set to auto, the figure will try to automatically determine the correct supplement based on the kind and the active text language. If you are using a custom figure type, you will need to manually specify the supplement.

If a function is specified, it is passed the first descendant of the specified kind (typically, the figure's body) and should return content.

Preview

Default: auto
numbering
none or str or function
Settable

How to number the figure. Accepts a numbering pattern or function taking a single number.

Default: "1"
gap
length
Settable

The vertical gap between the body and caption.

Default: 0.65em
outlined
bool
Settable

Whether the figure should appear in an outline of figures.

Default: true
Definitions
caption
Element

The caption of a figure. This element can be used in set and show rules to customize the appearance of captions for all figures or figures of a specific kind.

In addition to its position and body, the caption also provides the figure's kind, supplement, counter, and numbering as fields. These parts can be used in where selectors and show rules to build a completely custom caption.

#show figure.caption: emph

#figure(
  rect[Hello],
  caption: [A rectangle],
)

Preview
figure.caption(
position: alignment,separator: autocontent,content,
) -> content
position
alignment
Settable

The caption's position in the figure. Either top or bottom.

Preview

Default: bottom
separator
auto or content
Settable

The separator which will appear between the number and body.

If set to auto, the separator will be adapted to the current language and region.

Preview

Default: auto
body
content
Required
Positional

The caption's body.

Can be used alongside kind, supplement, counter, numbering, and location to completely customize the caption.

Preview



    Docs
    Reference
    Model
    Heading

heading
Element

A section heading.

With headings, you can structure your document into sections. Each heading has a level, which starts at one and is unbounded upwards. This level indicates the logical role of the following content (section, subsection, etc.) A top-level heading indicates a top-level section of the document (not the document's title). To insert a title, use the title element instead.

Typst can automatically number your headings for you. To enable numbering, specify how you want your headings to be numbered with a numbering pattern or function.

Independently of the numbering, Typst can also automatically generate an outline of all headings for you. To exclude one or more headings from this outline, you can set the outlined parameter to false.

When writing a show rule that accesses the body field to create a completely custom look for headings, make sure to wrap the content in a block (which is implicitly sticky for headings through a built-in show-set rule). This prevents headings from becoming "orphans", i.e. remaining at the end of the page with the following content being on the next page.
Example

#set heading(numbering: "1.a)")

= Introduction
In recent years, ...

== Preliminaries
To start, ...

Preview
Syntax

Headings have dedicated syntax: They can be created by starting a line with one or multiple equals signs, followed by a space. The number of equals signs determines the heading's logical nesting depth. The offset field can be set to configure the starting depth.
Accessibility

Headings are important for accessibility, as they help users of Assistive Technologies (AT) like screen readers to navigate within your document. Screen reader users will be able to skip from heading to heading, or get an overview of all headings in the document.

To make your headings accessible, you should not skip heading levels. This means that you should start with a first-level heading. Also, when the previous heading was of level 3, the next heading should be of level 3 (staying at the same depth), level 4 (going exactly one level deeper), or level 1 or 2 (new hierarchically higher headings).
HTML export

As mentioned above, a top-level heading indicates a top-level section of the document rather than its title. This is in contrast to the HTML <h1> element of which there should be only one per document.

For this reason, in HTML export, a title element will turn into an <h1> and headings turn into <h2> and lower (a level 1 heading thus turns into <h2>, a level 2 heading into <h3>, etc).
Parameters
heading(
level: autoint,depth: int,offset: int,numbering: nonestrfunction,supplement: noneautocontentfunction,outlined: bool,bookmarked: autobool,hanging-indent: autolength,content,
) -> content
level
auto or int
Settable

The absolute nesting depth of the heading, starting from one. If set to auto, it is computed from offset + depth.

This is primarily useful for usage in show rules (either with where selectors or by accessing the level directly on a shown heading).

Preview

Default: auto
depth
int
Settable

The relative nesting depth of the heading, starting from one. This is combined with offset to compute the actual level.

This is set by the heading syntax, such that == Heading creates a heading with logical depth of 2, but actual level offset + 2. If you construct a heading manually, you should typically prefer this over setting the absolute level.

Default: 1
offset
int
Settable

The starting offset of each heading's level, used to turn its relative depth into its absolute level.

Preview

Default: 0
numbering
none or str or function
Settable

How to number the heading. Accepts a numbering pattern or function taking multiple numbers.

Preview

Default: none
supplement
none or auto or content or function
Settable

A supplement for the heading.

For references to headings, this is added before the referenced number.

If a function is specified, it is passed the referenced heading and should return content.

Preview

Default: auto
outlined
bool
Settable

Whether the heading should appear in the outline.

Note that this property, if set to true, ensures the heading is also shown as a bookmark in the exported PDF's outline (when exporting to PDF). To change that behavior, use the bookmarked property.

Preview

Default: true
bookmarked
auto or bool
Settable

Whether the heading should appear as a bookmark in the exported PDF's outline. Doesn't affect other export formats, such as PNG.

The default value of auto indicates that the heading will only appear in the exported PDF's outline if its outlined property is set to true, that is, if it would also be listed in Typst's outline. Setting this property to either true (bookmark) or false (don't bookmark) bypasses that behavior.

Preview

Default: auto
hanging-indent
auto or length
Settable

The indent all but the first line of a heading should have.

The default value of auto uses the width of the numbering as indent if the heading is aligned at the start of the text direction, and no indent for center and other alignments.

Preview

Default: auto
body
content
Required
Positional

The heading's title.



    Docs
    Reference
    Model
    Numbering

numbering

Applies a numbering to a sequence of numbers.

A numbering defines how a sequence of numbers should be displayed as content. It is defined either through a pattern string or an arbitrary function.

A numbering pattern consists of counting symbols, for which the actual number is substituted, their prefixes, and one suffix. The prefixes and the suffix are displayed as-is.
Example

#numbering("1.1)", 1, 2, 3) \
#numbering("1.a.i", 1, 2) \
#numbering("I – 1", 12, 2) \
#numbering(
  (..nums) => nums
    .pos()
    .map(str)
    .join(".") + ")",
  1, 2, 3,
)

Preview
Numbering patterns and numbering functions

There are multiple instances where you can provide a numbering pattern or function in Typst. For example, when defining how to number headings or figures. Every time, the expected format is the same as the one described below for the numbering parameter.

The following example illustrates that a numbering function is just a regular function that accepts numbers and returns content.

#let unary(.., last) = "|" * last
#set heading(numbering: unary)
= First heading
= Second heading
= Third heading

Preview
Parameters
numbering(
strfunction,..int,
) -> any
numbering
str or function
Required
Positional

Defines how the numbering works.

Counting symbols are 1, a, A, i, I, α, Α, 一, 壹, あ, い, ア, イ, א, 가, ㄱ, *, ١, ۱, १, ১, ক, ①, and ⓵. They are replaced by the number in the sequence, preserving the original case.

The * character means that symbols should be used to count, in the order of *, †, ‡, §, ¶, ‖. If there are more than six items, the number is represented using repeated symbols.

Suffixes are all characters after the last counting symbol. They are displayed as-is at the end of any rendered number.

Prefixes are all characters that are neither counting symbols nor suffixes. They are displayed as-is at in front of their rendered equivalent of their counting symbol.

This parameter can also be an arbitrary function that gets each number as an individual argument. When given a function, the numbering function just forwards the arguments to that function. While this is not particularly useful in itself, it means that you can just give arbitrary numberings to the numbering function without caring whether they are defined as a pattern or function.
numbers
int
Required
Positional
Variadic

The numbers to apply the numbering to. Must be non-negative.

In general, numbers are counted from one. A number of zero indicates that the first element has not yet appeared.

If numbering is a pattern and more numbers than counting symbols are given, the last counting symbol with its prefix is repeated.



    Docs
    Reference
    Model
    Outline

outline
Element

A table of contents, figures, or other elements.

This function generates a list of all occurrences of an element in the document, up to a given depth. The element's numbering and page number will be displayed in the outline alongside its title or caption.
Example

#set heading(numbering: "1.")
#outline()

= Introduction
#lorem(5)

= Methods
== Setup
#lorem(10)

Preview
Alternative outlines

In its default configuration, this function generates a table of contents. By setting the target parameter, the outline can be used to generate a list of other kinds of elements than headings.

In the example below, we list all figures containing images by setting target to figure.where(kind: image). Just the same, we could have set it to figure.where(kind: table) to generate a list of tables.

We could also set it to just figure, without using a where selector, but then the list would contain all figures, be it ones containing images, tables, or other material.

#outline(
  title: [List of Figures],
  target: figure.where(kind: image),
)

#figure(
  image("tiger.jpg"),
  caption: [A nice figure!],
)

Preview
Styling the outline

At the most basic level, you can style the outline by setting properties on it and its entries. This way, you can customize the outline's title, how outline entries are indented, and how the space between an entry's text and its page number should be filled.

Richer customization is possible through configuration of the outline's entries. The outline generates one entry for each outlined element.
Spacing the entries

Outline entries are blocks, so you can adjust the spacing between them with normal block-spacing rules:

#show outline.entry.where(
  level: 1
): set block(above: 1.2em)

#outline()

= About ACME Corp.
== History
=== Origins
= Products
== ACME Tools

Preview
Building an outline entry from its parts

For full control, you can also write a transformational show rule on outline.entry. However, the logic for properly formatting and indenting outline entries is quite complex and the outline entry itself only contains two fields: The level and the outlined element.

For this reason, various helper functions are provided. You can mix and match these to compose an entry from just the parts you like.

The default show rule for an outline entry looks like this1:

#show outline.entry: it => link(
  it.element.location(),
  it.indented(it.prefix(), it.inner()),
)

    The indented function takes an optional prefix and inner content and automatically applies the proper indentation to it, such that different entries align nicely and long headings wrap properly.

    The prefix function formats the element's numbering (if any). It also appends a supplement for certain elements.

    The inner function combines the element's body, the filler, and the page number.

You can use these individual functions to format the outline entry in different ways. Let's say, you'd like to fully remove the filler and page numbers. To achieve this, you could write a show rule like this:

#show outline.entry: it => link(
  it.element.location(),
  // Keep just the body, dropping
  // the fill and the page.
  it.indented(it.prefix(), it.body()),
)

#outline()

= About ACME Corp.
== History

Preview
1

The outline of equations is the exception to this rule as it does not have a body and thus does not use indented layout.
Parameters
outline(
title: noneautocontent,target: labelselectorlocationfunction,depth: noneint,indent: autorelativefunction,
) -> content
title
none or auto or content
Settable

The title of the outline.

    When set to auto, an appropriate title for the text language will be used.
    When set to none, the outline will not have a title.
    A custom title can be set by passing content.

The outline's heading will not be numbered by default, but you can force it to be with a show-set rule: show outline: set heading(numbering: "1.")

Default: auto
target
label or selector or location or function
Settable

The type of element to include in the outline.

To list figures containing a specific kind of element, like an image or a table, you can specify the desired kind in a where selector. See the section on alternative outlines for more details.

Preview

Default: heading
depth
none or int
Settable

The maximum level up to which elements are included in the outline. When this argument is none, all elements are included.

Preview

Default: none
indent
auto or relative or function
Settable

How to indent the outline's entries.

    auto: Indents the numbering/prefix of a nested entry with the title of its parent entry. If the entries are not numbered (e.g., via heading numbering), this instead simply inserts a fixed amount of 1.2em indent per level.

    Relative length: Indents the entry by the specified length per nesting level. Specifying 2em, for instance, would indent top-level headings by 0em (not nested), second level headings by 2em (nested once), third-level headings by 4em (nested twice) and so on.

    Function: You can further customize this setting with a function. That function receives the nesting level as a parameter (starting at 0 for top-level headings/elements) and should return a (relative) length. For example, n => n * 2em would be equivalent to just specifying 2em.

Preview

Default: auto
Definitions
entry
Element

Represents an entry line in an outline.

With show-set and show rules on outline entries, you can richly customize the outline's appearance. See the section on styling the outline for details.
outline.entry(
int,content,fill: nonecontent,
) -> content
level
int
Required
Positional

The nesting level of this outline entry. Starts at 1 for top-level entries.
element
content
Required
Positional

The element this entry refers to. Its location will be available through the location method on the content and can be linked to.
fill
none or content
Settable

Content to fill the space between the title and the page number. Can be set to none to disable filling.

The fill will be placed into a fractionally sized box that spans the space between the entry's body and the page number. When using show rules to override outline entries, it is thus recommended to wrap the fill in a box with fractional width, i.e. box(width: 1fr, it.fill).

When using repeat, the gap property can be useful to tweak the visual weight of the fill.

Preview

Default: repeat(body: [.], gap: 0.15em)
Definitions of entry
indented
Contextual

A helper function for producing an indented entry layout: Lays out a prefix and the rest of the entry in an indent-aware way.

If the parent outline's indent is auto, the inner content of all entries at level N is aligned with the prefix of all entries at level N + 1, leaving at least gap space between the prefix and inner parts. Furthermore, the inner contents of all entries at the same level are aligned.

If the outline's indent is a fixed value or a function, the prefixes are indented, but the inner contents are simply offset from the prefix by the specified gap, rather than aligning outline-wide. For a visual explanation, see outline.indent.
self.indented(
nonecontent,content,gap: length,
) -> content
prefix
none or content
Required
Positional

The prefix is aligned with the inner content of entries that have level one less.

In the default show rule, this is just it.prefix(), but it can be freely customized.
inner
content
Required
Positional

The formatted inner content of the entry.

In the default show rule, this is just it.inner(), but it can be freely customized.
gap
length

The gap between the prefix and the inner content.

Default: 0.5em
prefix
Contextual

Formats the element's numbering (if any).

This also appends the element's supplement in case of figures or equations. For instance, it would output 1.1 for a heading, but Figure 1 for a figure, as is usual for outlines.
self.prefix(
) -> nonecontent
inner
Contextual

Creates the default inner content of the entry.

This includes the body, the fill, and page number.
self.inner(
) -> content
body

The content which is displayed in place of the referred element at its entry in the outline. For a heading, this is its body; for a figure a caption and for equations, it is empty.
self.body(
) -> content
page
Contextual

The page number of this entry's element, formatted with the numbering set for the referenced page.
self.page(
) -> content