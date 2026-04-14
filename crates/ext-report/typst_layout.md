
    Docs
    Reference
    Layout
    Page

page
Element

Layouts its child onto one or multiple pages.

Although this function is primarily used in set rules to affect page properties, it can also be used to explicitly render its argument onto a set of pages of its own.

Pages can be set to use auto as their width or height. In this case, the pages will grow to fit their content on the respective axis.

The Guide for Page Setup explains how to use this and related functions to set up a document with many examples.
Example

#set page("us-letter")

There you go, US friends!

Preview
Accessibility

The contents of the page's header, footer, foreground, and background are invisible to Assistive Technology (AT) like screen readers. Only the body of the page is read by AT. Do not include vital information not included elsewhere in the document in these areas.
Parameters
page(
paper: str,width: autolength,height: autolength,flipped: bool,margin: autorelativedictionary,binding: autoalignment,columns: int,fill: noneautocolorgradienttiling,numbering: nonestrfunction,supplement: noneautocontent,number-align: alignment,header: noneautocontent,header-ascent: relative,footer: noneautocontent,footer-descent: relative,background: nonecontent,foreground: nonecontent,body: content,
) -> content
paper
str

A standard paper size to set width and height.

This is just a shorthand for setting width and height and, as such, cannot be retrieved in a context expression.

Default: "a4"
width
auto or length
Settable

The width of the page.

Preview

Default: 595.28pt
height
auto or length
Settable

The height of the page.

If this is set to auto, page breaks can only be triggered manually by inserting a page break or by adding another non-empty page set rule. Most examples throughout this documentation use auto for the height of the page to dynamically grow and shrink to fit their content.

Default: 841.89pt
flipped
bool
Settable

Whether the page is flipped into landscape orientation.

Preview

Default: false
margin
auto or relative or dictionary
Settable

The page's margins.

    auto: The margins are set automatically to 2.5/21 times the smaller dimension of the page. This results in 2.5 cm margins for an A4 page.
    A single length: The same margin on all sides.
    A dictionary: With a dictionary, the margins can be set individually. The dictionary can contain the following keys in order of precedence:
        top: The top margin.
        right: The right margin.
        bottom: The bottom margin.
        left: The left margin.
        inside: The margin at the inner side of the page (where the binding is).
        outside: The margin at the outer side of the page (opposite to the binding).
        x: The horizontal margins.
        y: The vertical margins.
        rest: The margins on all sides except those for which the dictionary explicitly sets a size.

All keys are optional; omitted keys will use their previously set value, or the default margin if never set. In addition, the values for left and right are mutually exclusive with the values for inside and outside.

Preview

Default: auto
binding
auto or alignment
Settable

On which side the pages will be bound.

    auto: Equivalent to left if the text direction is left-to-right and right if it is right-to-left.
    left: Bound on the left side.
    right: Bound on the right side.

This affects the meaning of the inside and outside options for margins.

Default: auto
columns
int
Settable

How many columns the page has.

If you need to insert columns into a page or other container, you can also use the columns function.

Preview

Default: 1
fill
none or auto or color or gradient or tiling
Settable

The page's background fill.

Setting this to something non-transparent instructs the printer to color the complete page. If you are considering larger production runs, it may be more environmentally friendly and cost-effective to source pre-dyed pages and not set this property.

When set to none, the background becomes transparent. Note that PDF pages will still appear with a (usually white) background in viewers, but they are actually transparent. (If you print them, no color is used for the background.)

The default of auto results in none for PDF output, and white for PNG and SVG.

Preview

Default: auto
numbering
none or str or function
Settable

How to number the pages. You can refer to the Page Setup Guide for customizing page numbers.

Accepts a numbering pattern or function taking one or two numbers:

    The first number is the current page number.
    The second number is the total number of pages. In a numbering pattern, the second number can be omitted. If a function is passed, it will receive one argument in the context of links or references, and two arguments when producing the visible page numbers.

These are logical numbers controlled by the page counter, and may thus not match the physical numbers. Specifically, they are the current and the final value of counter(page). See the counter documentation for more details.

If an explicit footer (or header for top-aligned numbering) is given, the numbering is ignored.

Preview Preview

Default: none
supplement
none or auto or content
Settable

A supplement for the pages.

For page references, this is added before the page number.

Preview

Default: auto
number-align
alignment
Settable

The alignment of the page numbering.

If the vertical component is top, the numbering is placed into the header and if it is bottom, it is placed in the footer. Horizon alignment is forbidden. If an explicit matching header or footer is given, the numbering is ignored.

Preview

Default: center + bottom
header
none or auto or content
Settable

The page's header. Fills the top margin of each page.

    Content: Shows the content as the header.
    auto: Shows the page number if a numbering is set and number-align is top.
    none: Suppresses the header.

Preview

Default: auto
header-ascent
relative
Settable

The amount the header is raised into the top margin.

Default: 30% + 0pt
footer
none or auto or content
Settable

The page's footer. Fills the bottom margin of each page.

    Content: Shows the content as the footer.
    auto: Shows the page number if a numbering is set and number-align is bottom.
    none: Suppresses the footer.

For just a page number, the numbering property typically suffices. If you want to create a custom footer but still display the page number, you can directly access the page counter.

Preview Preview

Default: auto
footer-descent
relative
Settable

The amount the footer is lowered into the bottom margin.

Default: 30% + 0pt
background
none or content
Settable

Content in the page's background.

This content will be placed behind the page's body. It can be used to place a background image or a watermark.

Preview

Default: none
foreground
none or content
Settable

Content in the page's foreground.

This content will overlay the page's body.

Preview

Default: none
body
content

The contents of the page(s).

Multiple pages will be created if the content does not fit on a single page. A new page with the page properties prior to the function invocation will be created after the body has been typeset.

Default: [] 


grid
Element

Arranges content in a grid.

The grid element allows you to arrange content in a grid. You can define the number of rows and columns, as well as the size of the gutters between them. There are multiple sizing modes for columns and rows that can be used to create complex layouts.

While the grid and table elements work very similarly, they are intended for different use cases and carry different semantics. The grid element is intended for presentational and layout purposes, while the table element is intended for, in broad terms, presenting multiple related data points. Set and show rules on one of these elements do not affect the other. Refer to the Accessibility Section to learn how grids and tables are presented to users of Assistive Technology (AT) like screen readers.
Sizing the tracks

A grid's sizing is determined by the track sizes specified in the arguments. There are multiple sizing parameters: columns, rows and gutter. Because each of the sizing parameters accepts the same values, we will explain them just once, here. Each sizing argument accepts an array of individual track sizes. A track size is either:

    auto: The track will be sized to fit its contents. It will be at most as large as the remaining space. If there is more than one auto track width, and together they claim more than the available space, the auto tracks will fairly distribute the available space among themselves.

    A fixed or relative length (e.g. 10pt or 20% - 1cm): The track will be exactly of this size.

    A fractional length (e.g. 1fr): Once all other tracks have been sized, the remaining space will be divided among the fractional tracks according to their fractions. For example, if there are two fractional tracks, each with a fraction of 1fr, they will each take up half of the remaining space.

To specify a single track, the array can be omitted in favor of a single value. To specify multiple auto tracks, enter the number of tracks instead of an array. For example, columns: 3 is equivalent to columns: (auto, auto, auto).
Examples

The example below demonstrates the different track sizing options. It also shows how you can use grid.cell to make an individual cell span two grid tracks.

// We use `rect` to emphasize the
// area of cells.
#set rect(
  inset: 8pt,
  fill: rgb("e4e5ea"),
  width: 100%,
)

#grid(
  columns: (60pt, 1fr, 2fr),
  rows: (auto, 60pt),
  gutter: 3pt,
  rect[Fixed width, auto height],
  rect[1/3 of the remains],
  rect[2/3 of the remains],
  rect(height: 100%)[Fixed height],
  grid.cell(
    colspan: 2,
    image("tiger.jpg", width: 100%),
  ),
)

Preview

You can also spread an array of strings or content into a grid to populate its cells.

#grid(
  columns: 5,
  gutter: 5pt,
  ..range(25).map(str)
)

Preview
Styling the grid

The grid and table elements work similarly. For a hands-on explanation, refer to the Table Guide; for a quick overview, continue reading.

The grid's appearance can be customized through different parameters. These are the most important ones:

    align to change how cells are aligned
    inset to optionally add internal padding to cells
    fill to give cells a background
    stroke to optionally enable grid lines with a certain stroke

To meet different needs, there are various ways to set them.

If you need to override the above options for individual cells, you can use the grid.cell element. Likewise, you can override individual grid lines with the grid.hline and grid.vline elements.

To configure an overall style for a grid, you may instead specify the option in any of the following fashions:

    As a single value that applies to all cells.
    As an array of values corresponding to each column. The array will be cycled if there are more columns than the array has items.
    As a function in the form of (x, y) => value. It receives the cell's column and row indices (both starting from zero) and should return the value to apply to that cell.

#grid(
  columns: 5,

  // By a single value
  align: center,
  // By a single but more complicated value
  inset: (x: 2pt, y: 3pt),
  // By an array of values (cycling)
  fill: (rgb("#239dad50"), none),
  // By a function that returns a value
  stroke: (x, y) => if calc.rem(x + y, 3) == 0 { 0.5pt },

  ..range(5 * 3).map(n => numbering("A", n + 1))
)

Preview

On top of that, you may apply styling rules to grid and grid.cell. Especially, the x and y fields of grid.cell can be used in a where selector, making it possible to style cells at specific columns or rows, or individual positions.
Stroke styling precedence

As explained above, there are three ways to set the stroke of a grid cell: through grid.cell's stroke field, by using grid.hline and grid.vline, or by setting the grid's stroke field. When multiple of these settings are present and conflict, the hline and vline settings take the highest precedence, followed by the cell settings, and finally the grid settings.

Furthermore, strokes of a repeated grid header or footer will take precedence over regular cell strokes.
Accessibility

Grids do not carry any special semantics. Assistive Technology (AT) does not offer the ability to navigate two-dimensionally by cell in grids. If you want to present tabular data, use the table element instead.

AT will read the grid cells in their semantic order. Usually, this is the order in which you passed them to the grid. However, if you manually positioned them using grid.cell's x and y arguments, cells will be read row by row, from left to right (in left-to-right documents). A cell will be read when its position is first reached.
Parameters
grid(
columns: autointrelativefractionarray,rows: autointrelativefractionarray,gutter: autointrelativefractionarray,column-gutter: autointrelativefractionarray,row-gutter: autointrelativefractionarray,inset: relativearraydictionaryfunction,align: autoarrayalignmentfunction,fill: nonecolorgradientarraytilingfunction,stroke: nonelengthcolorgradientarraystroketilingdictionaryfunction,..content,
) -> content
columns
auto or int or relative or fraction or array
Settable

The column sizes.

Either specify a track size array or provide an integer to create a grid with that many auto-sized columns. Note that opposed to rows and gutters, providing a single track size will only ever create a single column.

See the track size section above for more details.

Default: ()
rows
auto or int or relative or fraction or array
Settable

The row sizes.

If there are more cells than fit the defined rows, the last row is repeated until there are no more cells.

See the track size section above for more details.

Default: ()
gutter
auto or int or relative or fraction or array

The gaps between rows and columns. This is a shorthand to set column-gutter and row-gutter to the same value.

If there are more gutters than defined sizes, the last gutter is repeated.

See the track size section above for more details.

Default: ()
column-gutter
auto or int or relative or fraction or array
Settable

The gaps between columns.

Default: ()
row-gutter
auto or int or relative or fraction or array
Settable

The gaps between rows.

Default: ()
inset
relative or array or dictionary or function
Settable

How much to pad the cells' content.

To specify a uniform inset for all cells, you can use a single length for all sides, or a dictionary of lengths for individual sides. See the box's documentation for more details.

To specify varying inset for different cells, you can:

    use a single inset for all cells
    use an array of insets corresponding to each column
    use a function that maps a cell's position to its inset

See the styling section above for more details.

In addition, you can find an example at the table.inset parameter.

Default: (:)
align
auto or array or alignment or function
Settable

How to align the cells' content.

If set to auto, the outer alignment is used.

You can specify the alignment in any of the following fashions:

    use a single alignment for all cells
    use an array of alignments corresponding to each column
    use a function that maps a cell's position to its alignment

See the styling section above for details.

In addition, you can find an example at the table.align parameter.

Default: auto
fill
none or color or gradient or array or tiling or function
Settable

How to fill the cells.

This can be:

    a single color for all cells
    an array of colors corresponding to each column
    a function that maps a cell's position to its color

Most notably, arrays and functions are useful for creating striped grids. See the styling section above for more details.

Preview

Default: none
stroke
none or length or color or gradient or array or stroke or tiling or dictionary or function
Settable

How to stroke the cells.

Grids have no strokes by default, which can be changed by setting this option to the desired stroke.

If it is necessary to place lines which can cross spacing between cells produced by the gutter option, or to override the stroke between multiple specific cells, consider specifying one or more of grid.hline and grid.vline alongside your grid cells.

To specify the same stroke for all cells, you can use a single stroke for all sides, or a dictionary of strokes for individual sides. See the rectangle's documentation for more details.

To specify varying strokes for different cells, you can:

    use a single stroke for all cells
    use an array of strokes corresponding to each column
    use a function that maps a cell's position to its stroke

See the styling section above for more details.

Preview

Preview Preview

Default: (:)
children
content
Required
Positional
Variadic

The contents of the grid cells, plus any extra grid lines specified with the grid.hline and grid.vline elements.

The cells are populated in row-major order.
Definitions
cell
Element

A cell in the grid. You can use this function in the argument list of a grid to override grid style properties for an individual cell or manually positioning it within the grid. You can also use this function in show rules to apply certain styles to multiple cells at once.

For example, you can override the position and stroke for a single cell:

#set text(15pt, font: "Noto Sans Symbols 2")
#show regex("[♚-♟︎]"): set text(fill: rgb("21212A"))
#show regex("[♔-♙]"): set text(fill: rgb("111015"))

#grid(
  fill: (x, y) => rgb(
    if calc.odd(x + y) { "7F8396" }
    else { "EFF0F3" }
  ),
  columns: (1em,) * 8,
  rows: 1em,
  align: center + horizon,

  [♖], [♘], [♗], [♕], [♔], [♗], [♘], [♖],
  [♙], [♙], [♙], [♙], [],  [♙], [♙], [♙],
  grid.cell(
    x: 4, y: 3,
    stroke: blue.transparentize(60%)
  )[♙],

  ..(grid.cell(y: 6)[♟],) * 8,
  ..([♜], [♞], [♝], [♛], [♚], [♝], [♞], [♜])
    .map(grid.cell.with(y: 7)),
)

Preview

You may also apply a show rule on grid.cell to style all cells at once, which allows you, for example, to apply styles based on a cell's position. Refer to the examples of the table.cell element to learn more about this.
grid.cell(
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

The cell's column (zero-indexed). This field may be used in show rules to style a cell depending on its column.

You may override this field to pick in which column the cell must be placed. If no row (y) is chosen, the cell will be placed in the first row (starting at row 0) with that column available (or a new row if none). If both x and y are chosen, however, the cell will be placed in that exact position. An error is raised if that position is not available (thus, it is usually wise to specify cells with a custom position before cells with automatic positions).

Preview

Default: auto
y
auto or int
Settable

The cell's row (zero-indexed). This field may be used in show rules to style a cell depending on its row.

You may override this field to pick in which row the cell must be placed. If no column (x) is chosen, the cell will be placed in the first column (starting at column 0) available in the chosen row. If all columns in the chosen row are already occupied, an error is raised.

Preview

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

A horizontal line in the grid.

Overrides any per-cell stroke, including stroke specified through the grid's stroke field. Can cross spacing between cells created through the grid's column-gutter option.

An example for this function can be found at the table.hline element.
grid.hline(
y: autoint,start: int,end: noneint,stroke: nonelengthcolorgradientstroketilingdictionary,position: alignment,
) -> content
y
auto or int
Settable

The row above which the horizontal line is placed (zero-indexed). If the position field is set to bottom, the line is placed below the row with the given index instead (see grid.hline.position for details).

Specifying auto causes the line to be placed at the row below the last automatically positioned cell (that is, cell without coordinate overrides) before the line among the grid's children. If there is no such cell before the line, it is placed at the top of the grid (row 0). Note that specifying for this option exactly the total amount of rows in the grid causes this horizontal line to override the bottom border of the grid, while a value of 0 overrides the top border.

Default: auto
start
int
Settable

The column at which the horizontal line starts (zero-indexed, inclusive).

Default: 0
end
none or int
Settable

The column before which the horizontal line ends (zero-indexed, exclusive). Therefore, the horizontal line will be drawn up to and across column end - 1.

A value equal to none or to the amount of columns causes it to extend all the way towards the end of the grid.

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

A vertical line in the grid.

Overrides any per-cell stroke, including stroke specified through the grid's stroke field. Can cross spacing between cells created through the grid's row-gutter option.
grid.vline(
x: autoint,start: int,end: noneint,stroke: nonelengthcolorgradientstroketilingdictionary,position: alignment,
) -> content
x
auto or int
Settable

The column before which the vertical line is placed (zero-indexed). If the position field is set to end, the line is placed after the column with the given index instead (see grid.vline.position for details).

Specifying auto causes the line to be placed at the column after the last automatically positioned cell (that is, cell without coordinate overrides) before the line among the grid's children. If there is no such cell before the line, it is placed before the grid's first column (column 0). Note that specifying for this option exactly the total amount of columns in the grid causes this vertical line to override the end border of the grid (right in LTR, left in RTL), while a value of 0 overrides the start border (left in LTR, right in RTL).

Default: auto
start
int
Settable

The row at which the vertical line starts (zero-indexed, inclusive).

Default: 0
end
none or int
Settable

The row on top of which the vertical line ends (zero-indexed, exclusive). Therefore, the vertical line will be drawn up to and across row end - 1.

A value equal to none or to the amount of rows causes it to extend all the way towards the bottom of the grid.

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

The values left and right are also accepted, but discouraged as they cause your grid to be inconsistent between left-to-right and right-to-left documents.

This setting is only relevant when column gutter is enabled (and shouldn't be used otherwise - prefer just increasing the x field by one instead), since then the position after a column becomes different from the position before the next column due to the spacing between both.

Default: start
header
Element

A repeatable grid header.

If repeat is set to true, the header will be repeated across pages. For an example, refer to the table.header element and the grid.stroke parameter.
grid.header(
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

A repeatable grid footer.

Just like the grid.header element, the footer can repeat itself on every page of the grid.

No other grid cells may be placed after the footer.
grid.footer(
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

block
Element

A block-level container.

Such a container can be used to separate content, size it, and give it a background or border.

Blocks are also the primary way to control whether text becomes part of a paragraph or not. See the paragraph documentation for more details.
Examples

With a block, you can give a background to content while still allowing it to break across multiple pages.

#set page(height: 100pt)
#block(
  fill: luma(230),
  inset: 8pt,
  radius: 4pt,
  lorem(30),
)

Preview Preview

Blocks are also useful to force elements that would otherwise be inline to become block-level, especially when writing show rules.

#show heading: it => it.body
= Blockless
More text.

#show heading: it => block(it.body)
= Blocky
More text.

Preview
Parameters
block(
width: autorelative,height: autorelativefraction,breakable: bool,fill: nonecolorgradienttiling,stroke: nonelengthcolorgradientstroketilingdictionary,radius: relativedictionary,inset: relativedictionary,outset: relativedictionary,spacing: relativefraction,above: autorelativefraction,below: autorelativefraction,clip: bool,sticky: bool,nonecontent,
) -> content
width
auto or relative
Settable

The block's width.

Preview

Default: auto
height
auto or relative or fraction
Settable

The block's height. When the height is larger than the remaining space on a page and breakable is true, the block will continue on the next page with the remaining height.

Preview Preview

Default: auto
breakable
bool
Settable

Whether the block can be broken and continue on the next page.

Preview Preview

Default: true
fill
none or color or gradient or tiling
Settable

The block's background color. See the rectangle's documentation for more details.

Default: none
stroke
none or length or color or gradient or stroke or tiling or dictionary
Settable

The block's border color. See the rectangle's documentation for more details.

Default: (:)
radius
relative or dictionary
Settable

How much to round the block's corners. See the rectangle's documentation for more details.

Default: (:)
inset
relative or dictionary
Settable

How much to pad the block's content. See the box's documentation for more details.

Default: (:)
outset
relative or dictionary
Settable

How much to expand the block's size without affecting the layout. See the box's documentation for more details.

Default: (:)
spacing
relative or fraction

The spacing around the block. When auto, inherits the paragraph spacing.

For two adjacent blocks, the larger of the first block's above and the second block's below spacing wins. Moreover, block spacing takes precedence over paragraph spacing.

Note that this is only a shorthand to set above and below to the same value. Since the values for above and below might differ, a context block only provides access to block.above and block.below, not to block.spacing directly.

This property can be used in combination with a show rule to adjust the spacing around arbitrary block-level elements.

Preview

Default: 1.2em
above
auto or relative or fraction
Settable

The spacing between this block and its predecessor.

Default: auto
below
auto or relative or fraction
Settable

The spacing between this block and its successor.

Default: auto
clip
bool
Settable

Whether to clip the content inside the block.

Clipping is useful when the block's content is larger than the block itself, as any content that exceeds the block's bounds will be hidden.

Preview

Default: false
sticky
bool
Settable

Whether this block must stick to the following one, with no break in between.

This is, by default, set on heading blocks to prevent orphaned headings at the bottom of the page.

Preview Preview

Default: false
body
none or content
Positional
Settable

The contents of the block.

Default: none


place
Element

Places content relatively to its parent container.

Placed content can be either overlaid (the default) or floating. Overlaid content is aligned with the parent container according to the given alignment, and shown over any other content added so far in the container. Floating content is placed at the top or bottom of the container, displacing other content down or up respectively. In both cases, the content position can be adjusted with dx and dy offsets without affecting the layout.

The parent can be any container such as a block, box, rect, etc. A top level place call will place content directly in the text area of the current page. This can be used for absolute positioning on the page: with a top + left alignment, the offsets dx and dy will set the position of the element's top left corner relatively to the top left corner of the text area. For absolute positioning on the full page including margins, you can use place in page.foreground or page.background.
Examples

#set page(height: 120pt)
Hello, world!

#rect(
  width: 100%,
  height: 2cm,
  place(horizon + right, square()),
)

#place(
  top + left,
  dx: -5pt,
  square(size: 5pt, fill: red),
)

Preview
Effect on the position of other elements

Overlaid elements don't take space in the flow of content, but a place call inserts an invisible block-level element in the flow. This can affect the layout by breaking the current paragraph. To avoid this, you can wrap the place call in a box when the call is made in the middle of a paragraph. The alignment and offsets will then be relative to this zero-size box. To make sure it doesn't interfere with spacing, the box should be attached to a word using a word joiner.

For example, the following defines a function for attaching an annotation to the following word:

#let annotate(..args) = {
  box(place(..args))
  sym.wj
  h(0pt, weak: true)
}

A placed #annotate(square(), dy: 2pt)
square in my text.

Preview

The zero-width weak spacing serves to discard spaces between the function call and the next word.
Accessibility

Assistive Technology (AT) will always read the placed element at the point where it logically appears in the document, regardless of where this function physically moved it. Put its markup where it would make the most sense in the reading order.
Parameters
place(
autoalignment,scope: str,float: bool,clearance: length,dx: relative,dy: relative,content,
) -> content
alignment
auto or alignment
Positional
Settable

Relative to which position in the parent container to place the content.

    If float is false, then this can be any alignment other than auto.
    If float is true, then this must be auto, top, or bottom.

When float is false and no vertical alignment is specified, the content is placed at the current position on the vertical axis.

Default: start
scope
str
Settable

Relative to which containing scope something is placed.

The parent scope is primarily used with figures and, for this reason, the figure function has a mirrored scope parameter. Nonetheless, it can also be more generally useful to break out of the columns. A typical example would be to create a single-column title section in a two-column document.

Note that parent-scoped placement is currently only supported if float is true. This may change in the future.

Preview
Variant	Details
"column"	

Place into the current column.
"parent"	

Place relative to the parent, letting the content span over all columns.

Default: "column"
float
bool
Settable

Whether the placed element has floating layout.

Floating elements are positioned at the top or bottom of the parent container, displacing in-flow content. They are always placed in the in-flow order relative to each other, as well as before any content following a later place.flush element.

Preview Preview

Default: false
clearance
length
Settable

The spacing between the placed element and other elements in a floating layout.

Has no effect if float is false.

Default: 1.5em
dx
relative
Settable

The horizontal displacement of the placed content.

Preview

This does not affect the layout of in-flow content. In other words, the placed content is treated as if it were wrapped in a move element.

Default: 0% + 0pt
dy
relative
Settable

The vertical displacement of the placed content.

This does not affect the layout of in-flow content. In other words, the placed content is treated as if it were wrapped in a move element.

Default: 0% + 0pt
body
content
Required
Positional

The content to place.
Definitions
flush
Element

Asks the layout algorithm to place pending floating elements before continuing with the content.

This is useful for preventing floating figures from spilling into the next section.

#lorem(15)

#figure(
  rect(width: 100%, height: 50pt),
  placement: auto,
  caption: [A rectangle],
)

#place.flush()

This text appears after the figure.

Preview Preview
place.flush(
) -> content



    Docs
    Reference
    Layout
    Padding

pad
Element

Adds spacing around content.

The spacing can be specified for each side individually, or for all sides at once by specifying a positional argument.
Example

#set align(center)

#pad(x: 16pt, image("typing.jpg"))
_Typing speeds can be
 measured in words per minute._

Preview
Parameters
pad(
left: relative,top: relative,right: relative,bottom: relative,x: relative,y: relative,rest: relative,content,
) -> content
left
relative
Settable

The padding at the left side.

Default: 0% + 0pt
top
relative
Settable

The padding at the top side.

Default: 0% + 0pt
right
relative
Settable

The padding at the right side.

Default: 0% + 0pt
bottom
relative
Settable

The padding at the bottom side.

Default: 0% + 0pt
x
relative

A shorthand to set left and right to the same value.

Default: 0% + 0pt
y
relative

A shorthand to set top and bottom to the same value.

Default: 0% + 0pt
rest
relative

A shorthand to set all four sides to the same value.

Default: 0% + 0pt
body
content
Required
Positional

The content to pad at the sides.



    Docs
    Reference
    Layout
    Stack

stack
Element

Arranges content and spacing horizontally or vertically.

The stack places a list of items along an axis, with optional spacing between each item.
Example

#stack(
  dir: ttb,
  rect(width: 40pt),
  rect(width: 120pt),
  rect(width: 90pt),
)

Preview
Accessibility

Stacks do not carry any special semantics. The contents of the stack are read by Assistive Technology (AT) in the order in which they have been passed to this function.
Parameters
stack(
dir: direction,spacing: nonerelativefraction,..relativefractioncontent,
) -> content
dir
direction
Settable

The direction along which the items are stacked. Possible values are:

    ltr: Left to right.
    rtl: Right to left.
    ttb: Top to bottom.
    btt: Bottom to top.

You can use the start and end methods to obtain the initial and final points (respectively) of a direction, as alignment. You can also use the axis method to determine whether a direction is "horizontal" or "vertical". The inv method returns a direction's inverse direction.

For example, ttb.start() is top, ttb.end() is bottom, ttb.axis() is "vertical" and ttb.inv() is equal to btt.

Default: ttb
spacing
none or relative or fraction
Settable

Spacing to insert between items where no explicit spacing was provided.

Default: none
children
relative or fraction or content
Required
Positional
Variadic

The children to stack along the axis.


    Docs
    Reference
    Layout
    Align

align
Element

Aligns content horizontally and vertically.
Example

Let's start with centering our content horizontally:

#set page(height: 120pt)
#set align(center)

Centered text, a sight to see \
In perfect balance, visually \
Not left nor right, it stands alone \
A work of art, a visual throne

Preview

To center something vertically, use horizon alignment:

#set page(height: 120pt)
#set align(horizon)

Vertically centered, \
the stage had entered, \
a new paragraph.

Preview
Combining alignments

You can combine two alignments with the + operator. Let's also only apply this to one piece of content by using the function form instead of a set rule:

#set page(height: 120pt)
Though left in the beginning ...

#align(right + bottom)[
  ... they were right in the end, \
  and with addition had gotten, \
  the paragraph to the bottom!
]

Preview
Nested alignment

You can use varying alignments for layout containers and the elements within them. This way, you can create intricate layouts:

#align(center, block[
  #set align(left)
  Though centered together \
  alone \
  we \
  are \
  left.
])

Preview
Alignment within the same line

The align function performs block-level alignment and thus always interrupts the current paragraph. To have different alignment for parts of the same line, you should use fractional spacing instead:

Start #h(1fr) End

Preview
Parameters
align(
alignment,content,
) -> content
alignment
alignment
Positional
Settable

The alignment along both axes.

Preview

Default: start + top
body
content
Required
Positional

The content to align.


    Docs
    Reference
    Layout
    Alignment

alignment

Where to align something along an axis.

Possible values are:

    start: Aligns at the start of the text direction.
    end: Aligns at the end of the text direction.
    left: Align at the left.
    center: Aligns in the middle, horizontally.
    right: Aligns at the right.
    top: Aligns at the top.
    horizon: Aligns in the middle, vertically.
    bottom: Align at the bottom.

These values are available globally and also in the alignment type's scope, so you can write either of the following two:

#align(center)[Hi]
#align(alignment.center)[Hi]

Preview
2D alignments

To align along both axes at the same time, add the two alignments using the + operator. For example, top + right aligns the content to the top right corner.

#set page(height: 3cm)
#align(center + bottom)[Hi]

Preview
Fields

The x and y fields hold the alignment's horizontal and vertical components, respectively (as yet another alignment). They may be none.

#(top + right).x \
#left.x \
#left.y (none)

Preview
Definitions
axis

The axis this alignment belongs to.

    "horizontal" for start, left, center, right, and end
    "vertical" for top, horizon, and bottom
    none for 2-dimensional alignments

#left.axis() \
#bottom.axis()

Preview
self.axis(
) -> nonestr
inv

The inverse alignment.

#top.inv() \
#left.inv() \
#center.inv() \
#(left + bottom).inv()

Preview
self.inv(
) -> alignment


    Docs
    Reference
    Layout
    Spacing (V)

v
Element

Inserts vertical spacing into a flow of blocks.

The spacing can be absolute, relative, or fractional. In the last case, the remaining space on the page is distributed among all fractional spacings according to their relative fractions.
Example

#grid(
  rows: 3cm,
  columns: 6,
  gutter: 1fr,
  [A #parbreak() B],
  [A #v(0pt) B],
  [A #v(10pt) B],
  [A #v(0pt, weak: true) B],
  [A #v(40%, weak: true) B],
  [A #v(1fr) B],
)

Preview
Parameters
v(
relativefraction,weak: bool,
) -> content
amount
relative or fraction
Required
Positional

How much spacing to insert.
weak
bool
Settable

If true, the spacing collapses at the start or end of a flow. Moreover, from multiple adjacent weak spacings all but the largest one collapse. Weak spacings will always collapse adjacent paragraph spacing, even if the paragraph spacing is larger.

Preview

Default: false


    Docs
    Reference
    Layout
    Spacing (H)

h
Element

Inserts horizontal spacing into a paragraph.

The spacing can be absolute, relative, or fractional. In the last case, the remaining space on the line is distributed among all fractional spacings according to their relative fractions.
Example

First #h(1cm) Second \
First #h(30%) Second

Preview
Fractional spacing

With fractional spacing, you can align things within a line without forcing a paragraph break (like align would). Each fractionally sized element gets space based on the ratio of its fraction to the sum of all fractions.

First #h(1fr) Second \
First #h(1fr) Second #h(1fr) Third \
First #h(2fr) Second #h(1fr) Third

Preview
Mathematical Spacing

In mathematical formulas, you can additionally use these constants to add spacing between elements: thin (1/6 em), med (2/9 em), thick (5/18 em), quad (1 em), wide (2 em).
Parameters
h(
relativefraction,weak: bool,
) -> content
amount
relative or fraction
Required
Positional

How much spacing to insert.
weak
bool
Settable

If true, the spacing collapses at the start or end of a paragraph. Moreover, from multiple adjacent weak spacings all but the largest one collapse.

Weak spacing in markup also causes all adjacent markup spaces to be removed, regardless of the amount of spacing inserted. To force a space next to weak spacing, you can explicitly write #" " (for a normal space) or ~ (for a non-breaking space). The latter can be useful to create a construct that always attaches to the preceding word with one non-breaking space, independently of whether a markup space existed in front or not.

Preview

Default: false


    Docs
    Reference
    Layout
    Columns

columns
Element

Separates a region into multiple equally sized columns.

The column function lets you separate the interior of any container into multiple columns. It will currently not balance the height of the columns. Instead, the columns will take up the height of their container or the remaining height on the page. Support for balanced columns is planned for the future.

When arranging content across multiple columns, use colbreak to explicitly continue in the next column.
Example

#columns(2, gutter: 8pt)[
  This text is in the
  first column.

  #colbreak()

  This text is in the
  second column.
]

Preview
Page-level columns

If you need to insert columns across your whole document, use the page function's columns parameter instead. This will create the columns directly at the page-level rather than wrapping all of your content in a layout container. As a result, things like pagebreaks, footnotes, and line numbers will continue to work as expected. For more information, also read the relevant part of the page setup guide.
Breaking out of columns

To temporarily break out of columns (e.g. for a paper's title), use parent-scoped floating placement:

#set page(columns: 2, height: 150pt)

#place(
  top + center,
  scope: "parent",
  float: true,
  text(1.4em, weight: "bold")[
    My document
  ],
)

#lorem(40)

Preview
Parameters
columns(
int,gutter: relative,content,
) -> content
count
int
Positional
Settable

The number of columns.

Default: 2
gutter
relative
Settable

The size of the gutter space between each column.

Default: 4% + 0pt
body
content
Required
Positional

The content that should be layouted into the columns.


    Docs
    Reference
    Layout
    Fraction

fraction

Defines how the remaining space in a layout is distributed.

Each fractionally sized element gets space based on the ratio of its fraction to the sum of all fractions.

For more details, also see the h and v functions and the grid function.
Example

Left #h(1fr) Left-ish #h(2fr) Right

Preview



    Docs
    Reference
    Layout
    Length

length

A size or distance, possibly expressed with contextual units.

Typst supports the following length units:

    Points: 72pt
    Millimeters: 254mm
    Centimeters: 2.54cm
    Inches: 1in
    Relative to font size: 2.5em

You can multiply lengths with and divide them by integers and floats.
Example

#rect(width: 20pt)
#rect(width: 2em)
#rect(width: 1in)

#(3em + 5pt).em \
#(20pt).em \
#(40em + 2pt).abs \
#(5em).abs

Preview
Fields

    abs: A length with just the absolute component of the current length (that is, excluding the em component).
    em: The amount of em units in this length, as a float.

Definitions
pt

Converts this length to points.

Fails with an error if this length has non-zero em units (such as 5em + 2pt instead of just 2pt). Use the abs field (such as in (5em + 2pt).abs.pt()) to ignore the em component of the length (thus converting only its absolute component).
self.pt(
) -> float
mm

Converts this length to millimeters.

Fails with an error if this length has non-zero em units. See the pt method for more details.
self.mm(
) -> float
cm

Converts this length to centimeters.

Fails with an error if this length has non-zero em units. See the pt method for more details.
self.cm(
) -> float
inches

Converts this length to inches.

Fails with an error if this length has non-zero em units. See the pt method for more details.
self.inches(
) -> float
to-absolute

Resolve this length to an absolute length.

#set text(size: 12pt)
#context [
  #(6pt).to-absolute() \
  #(6pt + 10em).to-absolute() \
  #(10em).to-absolute()
]

#set text(size: 6pt)
#context [
  #(6pt).to-absolute() \
  #(6pt + 10em).to-absolute() \
  #(10em).to-absolute()
]

Preview
self.to-absolute(
) -> length 


    Docs
    Reference
    Layout
    Relative Length

relative

A length in relation to some known length.

This type is a combination of a length with a ratio. It results from addition and subtraction of a length and a ratio. Wherever a relative length is expected, you can also use a bare length or ratio.
Relative to the page

A common use case is setting the width or height of a layout element (e.g., block, rect, etc.) as a certain percentage of the width of the page. Here, the rectangle's width is set to 25%, so it takes up one fourth of the page's inner width (the width minus margins).

#rect(width: 25%)

Preview

Bare lengths or ratios are always valid where relative lengths are expected, but the two can also be freely mixed:

#rect(width: 25% + 1cm)

Preview

If you're trying to size an element so that it takes up the page's full width, you have a few options (this highly depends on your exact use case):

    Set page margins to 0pt (#set page(margin: 0pt))
    Multiply the ratio by the known full page width (21cm * 69%)
    Use padding which will negate the margins (#pad(x: -2.5cm, ...))
    Use the page background or foreground field as those don't take margins into account (note that it will render the content outside of the document flow, see place to control the content position)

Relative to a container

When a layout element (e.g. a rect) is nested in another layout container (e.g. a block) instead of being a direct descendant of the page, relative widths become relative to the container:

#block(
  width: 100pt,
  fill: aqua,
  rect(width: 50%),
)

Preview
Scripting

You can multiply relative lengths by ratios, integers, and floats.

A relative length has the following fields:

    length: Its length component.
    ratio: Its ratio component.

#(100% - 50pt).length \
#(100% - 50pt).ratio

Preview