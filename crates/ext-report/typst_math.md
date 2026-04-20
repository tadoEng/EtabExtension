Math
Typst has special syntax and library functions to typeset mathematical formulas. Math formulas can be displayed inline with text or as separate blocks. They will be typeset into their own block if they start and end with at least one space (e.g. $ x^2 $).

Variables
In math, single letters are always displayed as is. Multiple letters, however, are interpreted as variables and functions. To display multiple letters verbatim, you can place them into quotes and to access single letter variables, you can use the hash syntax.

$ A = pi r^2 $
$ "area" = pi dot "radius"^2 $
$ cal(A) :=
    { x in RR | x "is natural" } $
#let x = 5
$ #x < 17 $
Preview
Symbols
Math mode makes a wide selection of symbols like pi, dot, or RR available. Many mathematical symbols are available in different variants. You can select between different variants by applying modifiers to the symbol. Typst further recognizes a number of shorthand sequences like => that approximate a symbol. When such a shorthand exists, the symbol's documentation lists it.

$ x < y => x gt.eq.not y $
Preview
Line Breaks
Formulas can also contain line breaks. Each line can contain one or multiple alignment points (&) which are then aligned.

$ sum_(k=0)^n k
    &= 1 + ... + n \
    &= (n(n+1)) / 2 $
Preview
Function calls
Math mode supports special function calls without the hash prefix. In these "math calls", the argument list works a little differently than in code:

Within them, Typst is still in "math mode". Thus, you can write math directly into them, but need to use hash syntax to pass code expressions (except for strings, which are available in the math syntax).
They support positional and named arguments, as well as argument spreading.
They don't support trailing content blocks.
They provide additional syntax for 2-dimensional argument lists. The semicolon (;) merges preceding arguments separated by commas into an array argument.
$ frac(a^2, 2) $
$ vec(1, 2, delim: "[") $
$ mat(1, 2; 3, 4) $
$ mat(..#range(1, 5).chunks(2)) $
$ lim_x =
    op("lim", limits: #true)_x $
Preview
To write a verbatim comma or semicolon in a math call, escape it with a backslash. The colon on the other hand is only recognized in a special way if directly preceded by an identifier, so to display it verbatim in those cases, you can just insert a space before it.

Functions calls preceded by a hash are normal code function calls and not affected by these rules.

Alignment
When equations include multiple alignment points (&), this creates blocks of alternatingly right- and left-aligned columns. In the example below, the expression (3x + y) / 7 is right-aligned and = 9 is left-aligned. The word "given" is also left-aligned because && creates two alignment points in a row, alternating the alignment twice. & & and && behave exactly the same way. Meanwhile, "multiply by 7" is right-aligned because just one & precedes it. Each alignment point simply alternates between right-aligned/left-aligned.

$ (3x + y) / 7 &= 9 && "given" \
  3x + y &= 63 & "multiply by 7" \
  3x &= 63 - y && "subtract y" \
  x &= 21 - y/3 & "divide by 3" $
Preview
Math fonts
You can set the math font by with a show-set rule as demonstrated below. Note that only special OpenType math fonts are suitable for typesetting maths.

#show math.equation: set text(font: "Fira Math")
$ sum_(i in NN) 1 + i $
Preview
Math module
All math functions are part of the math module, which is available by default in equations. Outside of equations, they can be accessed with the math. prefix.

Accessibility
To make math accessible, you must provide alternative descriptions of equations in natural language using the alt parameter of math.equation. For more information, see the Textual Representations section of the Accessibility Guide.

#math.equation(
  alt: "d S equals delta q divided by T",
  block: true,
  $ dif S = (delta q) / T $,
)
Preview
In the future, Typst will automatically make equations without alternative descriptions accessible in HTML and PDF 2.0 export.

Definitions
accent
Attaches an accent to a base.
attach
Subscript, superscripts, and limits.
binom
A binomial expression.
cancel
Displays a diagonal line over a part of an equation.
cases
A case distinction.
class
Forced use of a certain math class.
equation
A mathematical equation.
frac
A mathematical fraction.
lr
Delimiter matching.
mat
A matrix.
op
A text operator in an equation.
primes
Grouped primes.
roots
Square and non-square roots.
sizes
Forced size styles for expressions within formulas.
stretch
Stretches a glyph.
styles
Alternate letterforms within formulas.
underover
Delimiters above or below parts of an equation.
variants
Alternate typefaces within formulas.
vec
A column vector.

cases
Element
A case distinction.

Content across different branches can be aligned with the & symbol.

Example
$ f(x, y) := cases(
  1 "if" (x dot y)/2 <= 0,
  2 "if" x "is even",
  3 "if" x in NN,
  4 "else",
) $
Preview
Parameters
math.cases(
delim: nonestrarraysymbol,
reverse: bool,
gap: relative,
..content,
) -> content
delim
none or str or array or symbol
Settable
The delimiter to use.

Can be a single character specifying the left delimiter, in which case the right delimiter is inferred. Otherwise, can be an array containing a left and a right delimiter.

Default: ("{", "}")

reverse
bool
Settable
Whether the direction of cases should be reversed.

Default: false

gap
relative
Settable
The gap between branches.

Default: 0% + 0.2em

children
content
Required
Positional
Variadic
The branches of the case distinction.


equation
Element
A mathematical equation.

Can be displayed inline with text or as a separate block. An equation becomes block-level through the presence of whitespace after the opening dollar sign and whitespace before the closing dollar sign.

Example
#set text(font: "New Computer Modern")

Let $a$, $b$, and $c$ be the side
lengths of right-angled triangle.
Then, we know that:
$ a^2 + b^2 = c^2 $

Prove by induction:
$ sum_(k=1)^n k = (n(n+1)) / 2 $
Preview
By default, block-level equations will not break across pages. This can be changed through show math.equation: set block(breakable: true).

Syntax
This function also has dedicated syntax: Write mathematical markup within dollar signs to create an equation. Starting and ending the equation with whitespace lifts it into a separate block that is centered horizontally. For more details about math syntax, see the main math page.

Parameters
math.equation(
block: bool,
numbering: nonestrfunction,
number-align: alignment,
supplement: noneautocontentfunction,
alt: nonestr,
content,
) -> content
block
bool
Settable
Whether the equation is displayed as a separate block.

Default: false

numbering
none or str or function
Settable
How to number block-level equations. Accepts a numbering pattern or function taking a single number.

Default: none

number-align
alignment
Settable
The alignment of the equation numbering.

By default, the alignment is end + horizon. For the horizontal component, you can use right, left, or start and end of the text direction; for the vertical component, you can use top, horizon, or bottom.

Default: end + horizon

supplement
none or auto or content or function
Settable
A supplement for the equation.

For references to equations, this is added before the referenced number.

If a function is specified, it is passed the referenced equation and should return content.

Default: auto

alt
none or str
Settable
An alternative description of the mathematical equation.

This should describe the full equation in natural language and will be made available to Assistive Technology. You can learn more in the Textual Representations section of the Accessibility Guide.

Default: none

body
content
Required
Positional
The contents of the equation.

frac
Element
A mathematical fraction.

Example
$ 1/2 < (x+1)/2 $
$ ((x+1)) / 2 = frac(a, b) $
Preview
Syntax
This function also has dedicated syntax: Use a slash to turn neighbouring expressions into a fraction. Multiple atoms can be grouped into a single expression using round grouping parentheses. Such parentheses are removed from the output, but you can nest multiple to force them.

Parameters
math.frac(
content,
content,
style: str,
) -> content
num
content
Required
Positional
The fraction's numerator.

denom
content
Required
Positional
The fraction's denominator.

style
str
Settable
How the fraction should be laid out.

Variant	Details
"vertical"	
Stacked numerator and denominator with a bar.

"skewed"	
Numerator and denominator separated by a slash.

"horizontal"	
Numerator and denominator placed inline and parentheses are not absorbed.

Default: "vertical"

Left/Right
Delimiter matching.

The lr function allows you to match two delimiters and scale them with the content they contain. While this also happens automatically for delimiters that match syntactically, lr allows you to match two arbitrary delimiters and control their size exactly. Apart from the lr function, Typst provides a few more functions that create delimiter pairings for absolute, ceiled, and floored values as well as norms.

To prevent a delimiter from being matched by Typst, and thus auto-scaled, escape it with a backslash. To instead disable auto-scaling completely, use set math.lr(size: 1em).

Example
$ [a, b/2] $
$ lr(]sum_(x=1)^n], size: #50%) x $
$ abs((x + y) / 2) $
$ \{ (x / y) \} $
#set math.lr(size: 1em)
$ { (a / b), a, b in (0; 1/2] } $
Preview
Functions
lr
Element
Scales delimiters.

While matched delimiters scale by default, this can be used to scale unmatched delimiters and to control the delimiter scaling more precisely.

math.lr(
size: relative,
content,
) -> content
size
relative
Settable
The size of the brackets, relative to the height of the wrapped content.

Default: 100% + 0pt

body
content
Required
Positional
The delimited content, including the delimiters.

mid
Element
Scales delimiters vertically to the nearest surrounding lr() group.

$ { x mid(|) sum_(i=1)^n w_i|f_i (x)| < 1 } $
Preview
math.mid(
content
) -> content
body
content
Required
Positional
The content to be scaled.

abs
Takes the absolute value of an expression.

$ abs(x/2) $
Preview
math.abs(
size: relative,
content,
) -> content
size
relative
The size of the brackets, relative to the height of the wrapped content.

body
content
Required
Positional
The expression to take the absolute value of.

norm
Takes the norm of an expression.

$ norm(x/2) $
Preview
math.norm(
size: relative,
content,
) -> content
size
relative
The size of the brackets, relative to the height of the wrapped content.

body
content
Required
Positional
The expression to take the norm of.

floor
Floors an expression.

$ floor(x/2) $
Preview
math.floor(
size: relative,
content,
) -> content
size
relative
The size of the brackets, relative to the height of the wrapped content.

body
content
Required
Positional
The expression to floor.

ceil
Ceils an expression.

$ ceil(x/2) $
Preview
math.ceil(
size: relative,
content,
) -> content
size
relative
The size of the brackets, relative to the height of the wrapped content.

body
content
Required
Positional
The expression to ceil.

round
Rounds an expression.

$ round(x/2) $
Preview
math.round(
size: relative,
content,
) -> content
size
relative
The size of the brackets, relative to the height of the wrapped content.

body
content
Required
Positional
The expression to round.

mat
Element
A matrix.

The elements of a row should be separated by commas, while the rows themselves should be separated by semicolons. The semicolon syntax merges preceding arguments separated by commas into an array. You can also use this special syntax of math function calls to define custom functions that take 2D data.

Content in cells can be aligned with the align parameter, or content in cells that are in the same row can be aligned with the & symbol.

Example
$ mat(
  1, 2, ..., 10;
  2, 2, ..., 10;
  dots.v, dots.v, dots.down, dots.v;
  10, 10, ..., 10;
) $
Preview
Parameters
math.mat(
delim: nonestrarraysymbol,
align: alignment,
augment: noneintdictionary,
gap: relative,
row-gap: relative,
column-gap: relative,
..array,
) -> content
delim
none or str or array or symbol
Settable
The delimiter to use.

Can be a single character specifying the left delimiter, in which case the right delimiter is inferred. Otherwise, can be an array containing a left and a right delimiter.

Default: ("(", ")")

align
alignment
Settable
The horizontal alignment that each cell should have.

Default: center

augment
none or int or dictionary
Settable
Draws augmentation lines in a matrix.

none: No lines are drawn.
A single number: A vertical augmentation line is drawn after the specified column number. Negative numbers start from the end.
A dictionary: With a dictionary, multiple augmentation lines can be drawn both horizontally and vertically. Additionally, the style of the lines can be set. The dictionary can contain the following keys:
hline: The offsets at which horizontal lines should be drawn. For example, an offset of 2 would result in a horizontal line being drawn after the second row of the matrix. Accepts either an integer for a single line, or an array of integers for multiple lines. Like for a single number, negative numbers start from the end.
vline: The offsets at which vertical lines should be drawn. For example, an offset of 2 would result in a vertical line being drawn after the second column of the matrix. Accepts either an integer for a single line, or an array of integers for multiple lines. Like for a single number, negative numbers start from the end.
stroke: How to stroke the line. If set to auto, takes on a thickness of 0.05 em and square line caps.
Default: none

gap
relative
The gap between rows and columns.

This is a shorthand to set row-gap and column-gap to the same value.

Default: 0% + 0pt

row-gap
relative
Settable
The gap between rows.

Default: 0% + 0.2em

column-gap
relative
Settable
The gap between columns.

Default: 0% + 0.5em

rows
array
Required
Positional
Variadic
An array of arrays with the rows of the matrix.

primes
Element
Grouped primes.

$ a'''_b = a^'''_b $
Preview
Syntax
This function has dedicated syntax: use apostrophes instead of primes. They will automatically attach to the previous element, moving superscripts to the next level.

Parameters
math.primes(
int
) -> content
count
int
Required
Positional
The number of grouped primes.

Roots
Square and non-square roots.

Example
$ sqrt(3 - 2 sqrt(2)) = sqrt(2) - 1 $
$ root(3, x) $
Preview
Functions
root
Element
A general root.

$ root(3, x) $
Preview
math.root(
nonecontent,
content,
) -> content
index
none or content
Positional
Settable
Which root of the radicand to take.

Default: none

radicand
content
Required
Positional
The expression to take the root of.

sqrt
A square root.

$ sqrt(3 - 2 sqrt(2)) = sqrt(2) - 1 $
Preview
math.sqrt(
content
) -> content
radicand
content
Required
Positional
The expression to take the square root of.

Sizes
Forced size styles for expressions within formulas.

These functions allow manual configuration of the size of equation elements to make them look as in a display/inline equation or as if used in a root or sub/superscripts.

Functions
display
Forced display style in math.

This is the normal size for block equations.

$sum_i x_i/2 = display(sum_i x_i/2)$
Preview
math.display(
content,
cramped: bool,
) -> content
body
content
Required
Positional
The content to size.

cramped
bool
Whether to impose a height restriction for exponents, like regular sub- and superscripts do.

Default: false

inline
Forced inline (text) style in math.

This is the normal size for inline equations.

$ sum_i x_i/2
    = inline(sum_i x_i/2) $
Preview
math.inline(
content,
cramped: bool,
) -> content
body
content
Required
Positional
The content to size.

cramped
bool
Whether to impose a height restriction for exponents, like regular sub- and superscripts do.

Default: false

script
Forced script style in math.

This is the smaller size used in powers or sub- or superscripts.

$sum_i x_i/2 = script(sum_i x_i/2)$
Preview
math.script(
content,
cramped: bool,
) -> content
body
content
Required
Positional
The content to size.

cramped
bool
Whether to impose a height restriction for exponents, like regular sub- and superscripts do.

Default: true

sscript
Forced second script style in math.

This is the smallest size, used in second-level sub- and superscripts (script of the script).

$sum_i x_i/2 = sscript(sum_i x_i/2)$
Preview
math.sscript(
content,
cramped: bool,
) -> content
body
content
Required
Positional
The content to size.

cramped
bool
Whether to impose a height restriction for exponents, like regular sub- and superscripts do.

Default: true

Styles
Alternate letterforms within formulas.

These functions are distinct from the text function because math fonts contain multiple variants of each letter.

Functions
upright
Upright (non-italic) font style in math.

$ upright(A) != A $
Preview
math.upright(
content
) -> content
body
content
Required
Positional
The content to style.

italic
Italic font style in math.

For roman letters and greek lowercase letters, this is already the default.

math.italic(
content
) -> content
body
content
Required
Positional
The content to style.

bold
Bold font style in math.

$ bold(A) := B^+ $
Preview
math.bold(
content
) -> content
body
content
Required
Positional
The content to style.

op
Element
A text operator in an equation.

Example
$ tan x = (sin x)/(cos x) $
$ op("custom",
     limits: #true)_(n->oo) n $
Preview
Predefined Operators
Typst predefines the operators arccos, arcsin, arctan, arg, cos, cosh, cot, coth, csc, csch, ctg, deg, det, dim, exp, gcd, lcm, hom, id, im, inf, ker, lg, lim, liminf, limsup, ln, log, max, min, mod, Pr, sec, sech, sin, sinc, sinh, sup, tan, tanh, tg and tr.

Parameters
math.op(
content,
limits: bool,
) -> content
text
content
Required
Positional
The operator's text.

limits
bool
Settable
Whether the operator should show attachments as limits in display mode.

Default: false

Under/Over
Delimiters above or below parts of an equation.

The braces and brackets further allow you to add an optional annotation below or above themselves.

Functions
underline
Element
A horizontal line under content.

$ underline(1 + 2 + ... + 5) $
Preview
math.underline(
content
) -> content
body
content
Required
Positional
The content above the line.

overline
Element
A horizontal line over content.

$ overline(1 + 2 + ... + 5) $
Preview
math.overline(
content
) -> content
body
content
Required
Positional
The content below the line.

underbrace
Element
A horizontal brace under content, with an optional annotation below.

$ underbrace(0 + 1 + dots.c + n, n + 1 "numbers") $
Preview
math.underbrace(
content,
nonecontent,
) -> content
body
content
Required
Positional
The content above the brace.

annotation
none or content
Positional
Settable
The optional content below the brace.

Default: none

overbrace
Element
A horizontal brace over content, with an optional annotation above.

$ overbrace(0 + 1 + dots.c + n, n + 1 "numbers") $
Preview
math.overbrace(
content,
nonecontent,
) -> content
body
content
Required
Positional
The content below the brace.

annotation
none or content
Positional
Settable
The optional content above the brace.

Default: none

underbracket
Element
A horizontal bracket under content, with an optional annotation below.

$ underbracket(0 + 1 + dots.c + n, n + 1 "numbers") $
Preview
math.underbracket(
content,
nonecontent,
) -> content
body
content
Required
Positional
The content above the bracket.

annotation
none or content
Positional
Settable
The optional content below the bracket.

Default: none

overbracket
Element
A horizontal bracket over content, with an optional annotation above.

$ overbracket(0 + 1 + dots.c + n, n + 1 "numbers") $
Preview
math.overbracket(
content,
nonecontent,
) -> content
body
content
Required
Positional
The content below the bracket.

annotation
none or content
Positional
Settable
The optional content above the bracket.

Default: none

underparen
Element
A horizontal parenthesis under content, with an optional annotation below.

$ underparen(0 + 1 + dots.c + n, n + 1 "numbers") $
Preview
math.underparen(
content,
nonecontent,
) -> content
body
content
Required
Positional
The content above the parenthesis.

annotation
none or content
Positional
Settable
The optional content below the parenthesis.

Default: none

overparen
Element
A horizontal parenthesis over content, with an optional annotation above.

$ overparen(0 + 1 + dots.c + n, n + 1 "numbers") $
Preview
math.overparen(
content,
nonecontent,
) -> content
body
content
Required
Positional
The content below the parenthesis.

annotation
none or content
Positional
Settable
The optional content above the parenthesis.

Default: none

undershell
Element
A horizontal tortoise shell bracket under content, with an optional annotation below.

$ undershell(0 + 1 + dots.c + n, n + 1 "numbers") $
Preview
math.undershell(
content,
nonecontent,
) -> content
body
content
Required
Positional
The content above the tortoise shell bracket.

annotation
none or content
Positional
Settable
The optional content below the tortoise shell bracket.

Default: none

overshell
Element
A horizontal tortoise shell bracket over content, with an optional annotation above.

$ overshell(0 + 1 + dots.c + n, n + 1 "numbers") $
Preview
math.overshell(
content,
nonecontent,
) -> content
body
content
Required
Positional
The content below the tortoise shell bracket.

annotation
none or content
Positional
Settable
The optional content above the tortoise shell bracket.

Default: none

Variants
Alternate typefaces within formulas.

These functions are distinct from the text function because math fonts contain multiple variants of each letter.

Functions
serif
Serif (roman) font style in math.

This is already the default.

math.serif(
content
) -> content
body
content
Required
Positional
The content to style.

sans
Sans-serif font style in math.

$ sans(A B C) $
Preview
math.sans(
content
) -> content
body
content
Required
Positional
The content to style.

frak
Fraktur font style in math.

$ frak(P) $
Preview
math.frak(
content
) -> content
body
content
Required
Positional
The content to style.

mono
Monospace font style in math.

$ mono(x + y = z) $
Preview
math.mono(
content
) -> content
body
content
Required
Positional
The content to style.

bb
Blackboard bold (double-struck) font style in math.

For uppercase latin letters, blackboard bold is additionally available through symbols of the form NN and RR.

$ bb(b) $
$ bb(N) = NN $
$ f: NN -> RR $
Preview
math.bb(
content
) -> content
body
content
Required
Positional
The content to style.

cal
Calligraphic (chancery) font style in math.

Let $cal(P)$ be the set of ...
Preview
This is the default calligraphic/script style for most math fonts. See scr for more on how to get the other style (roundhand).

math.cal(
content
) -> content
body
content
Required
Positional
The content to style.

scr
Script (roundhand) font style in math.

$scr(L)$ is not the set of linear
maps $cal(L)$.
Preview
There are two ways that fonts can support differentiating cal and scr. The first is using Unicode variation sequences. This works out of the box in Typst, however only a few math fonts currently support this.

The other way is using font features. For example, the roundhand style might be available in a font through the stylistic set 1 (ss01) feature. To use it in Typst, you could then define your own version of scr like in the example below.

math.scr(
content
) -> content
body
content
Required
Positional
The content to style.

vec
Element
A column vector.

Content in the vector's elements can be aligned with the align parameter, or the & symbol.

This function is for typesetting vector components. To typeset a symbol that represents a vector, arrow and bold are commonly used.

Example
$ vec(a, b, c) dot vec(1, 2, 3)
    = a + 2b + 3c $
Preview
Parameters
math.vec(
delim: nonestrarraysymbol,
align: alignment,
gap: relative,
..content,
) -> content
delim
none or str or array or symbol
Settable
The delimiter to use.

Can be a single character specifying the left delimiter, in which case the right delimiter is inferred. Otherwise, can be an array containing a left and a right delimiter.

Default: ("(", ")")

align
alignment
Settable
The horizontal alignment that each element should have.

Default: center

gap
relative
Settable
The gap between elements.

Default: 0% + 0.2em

children
content
Required
Positional
Variadic
The elements of the vector.

