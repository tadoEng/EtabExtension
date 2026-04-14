
    Docs
    Reference
    Data Loading
    JSON

json

Reads structured data from a JSON file.

The file must contain a valid JSON value, such as object or array. The JSON values will be converted into corresponding Typst values as listed in the table below.

The function returns a dictionary, an array or, depending on the JSON file, another JSON data type.

The JSON files in the example contain objects with the keys temperature, unit, and weather.
Example

#let forecast(day) = block[
  #box(square(
    width: 2cm,
    inset: 8pt,
    fill: if day.weather == "sunny" {
      yellow
    } else {
      aqua
    },
    align(
      bottom + right,
      strong(day.weather),
    ),
  ))
  #h(6pt)
  #set text(22pt, baseline: -8pt)
  #day.temperature °#day.unit
]

#forecast(json("monday.json"))
#forecast(json("tuesday.json"))

Preview
Conversion details
JSON value	Converted into Typst
null	none
bool	bool
number	float or int
string	str
array	array
object	dictionary
Typst value	Converted into JSON
types that can be converted from JSON	corresponding JSON value
bytes	string via repr
symbol	string
content	an object describing the content
other types (length, etc.)	string via repr
Notes

    In most cases, JSON numbers will be converted to floats or integers depending on whether they are whole numbers. However, be aware that integers larger than 263-1 or smaller than -263 will be converted to floating-point numbers, which may result in an approximative value.

    Bytes are not encoded as JSON arrays for performance and readability reasons. Consider using cbor.encode for binary data.

    The repr function is for debugging purposes only, and its output is not guaranteed to be stable across Typst versions.

Parameters
json(
str
bytes
) -> any
source
str or bytes
Required
Positional

A path to a JSON file or raw JSON bytes.
Definitions
decode
json.decode is deprecated, directly pass bytes to json instead; it will be removed in Typst 0.15.0

Reads structured data from a JSON string/bytes.
json.decode(
str
bytes
) -> any
data
str or bytes
Required
Positional

JSON data.
encode

Encodes structured data into a JSON string.
json.encode(
any,pretty: bool,
) -> str
value
any
Required
Positional

Value to be encoded.
pretty
bool

Whether to pretty print the JSON with newlines and indentation.

Default: true



    Docs
    Reference
    Data Loading
    TOML

toml

Reads structured data from a TOML file.

The file must contain a valid TOML table. The TOML values will be converted into corresponding Typst values as listed in the table below.

The function returns a dictionary representing the TOML table.

The TOML file in the example consists of a table with the keys title, version, and authors.
Example

#let details = toml("details.toml")

Title: #details.title \
Version: #details.version \
Authors: #(details.authors
  .join(", ", last: " and "))

Preview
Conversion details

First of all, TOML documents are tables. Other values must be put in a table to be encoded or decoded.
TOML value	Converted into Typst
string	str
integer	int
float	float
boolean	bool
datetime	datetime
array	array
table	dictionary
Typst value	Converted into TOML
types that can be converted from TOML	corresponding TOML value
none	ignored
bytes	string via repr
symbol	string
content	a table describing the content
other types (length, etc.)	string via repr
Notes

    Be aware that TOML integers larger than 263-1 or smaller than -263 cannot be represented losslessly in Typst, and an error will be thrown according to the specification.

    Bytes are not encoded as TOML arrays for performance and readability reasons. Consider using cbor.encode for binary data.

    The repr function is for debugging purposes only, and its output is not guaranteed to be stable across Typst versions.

Parameters
toml(
str
bytes
) -> dictionary
source
str or bytes
Required
Positional

A path to a TOML file or raw TOML bytes.
Definitions
decode
toml.decode is deprecated, directly pass bytes to toml instead; it will be removed in Typst 0.15.0

Reads structured data from a TOML string/bytes.
toml.decode(
str
bytes
) -> dictionary
data
str or bytes
Required
Positional

TOML data.
encode

Encodes structured data into a TOML string.
toml.encode(
dictionary,pretty: bool,
) -> str
value
dictionary
Required
Positional

Value to be encoded.

TOML documents are tables. Therefore, only dictionaries are suitable.
pretty
bool

Whether to pretty-print the resulting TOML.

Default: true



    Docs
    Reference
    Data Loading
    CSV

csv

Reads structured data from a CSV file.

The CSV file will be read and parsed into a 2-dimensional array of strings: Each row in the CSV file will be represented as an array of strings, and all rows will be collected into a single array. Header rows will not be stripped.
Example

#let results = csv("example.csv")

#table(
  columns: 2,
  [*Condition*], [*Result*],
  ..results.flatten(),
)

Preview
Parameters
csv(
strbytes,delimiter: str,row-type: type,
) -> array
source
str or bytes
Required
Positional

A path to a CSV file or raw CSV bytes.
delimiter
str

The delimiter that separates columns in the CSV file. Must be a single ASCII character.

Default: ","
row-type
type

How to represent the file's rows.

    If set to array, each row is represented as a plain array of strings.
    If set to dictionary, each row is represented as a dictionary mapping from header keys to strings. This option only makes sense when a header row is present in the CSV file.

Default: array
Definitions
decode
csv.decode is deprecated, directly pass bytes to csv instead; it will be removed in Typst 0.15.0

Reads structured data from a CSV string/bytes.
csv.decode(
strbytes,delimiter: str,row-type: type,
) -> array
data
str or bytes
Required
Positional

CSV data.
delimiter
str

The delimiter that separates columns in the CSV file. Must be a single ASCII character.

Default: ","
row-type
type

How to represent the file's rows.

    If set to array, each row is represented as a plain array of strings.
    If set to dictionary, each row is represented as a dictionary mapping from header keys to strings. This option only makes sense when a header row is present in the CSV file.

Default: array



    Docs
    Reference
    Introspection
    Counter

counter

Counts through pages, elements, and more.

With the counter function, you can access and modify counters for pages, headings, figures, and more. Moreover, you can define custom counters for other things you want to count.

Since counters change throughout the course of the document, their current value is contextual. It is recommended to read the chapter on context before continuing here.
Accessing a counter

To access the raw value of a counter, we can use the get function. This function returns an array: Counters can have multiple levels (in the case of headings for sections, subsections, and so on), and each item in the array corresponds to one level.

#set heading(numbering: "1.")

= Introduction
Raw value of heading counter is
#context counter(heading).get()

Preview
Displaying a counter

Often, we want to display the value of a counter in a more human-readable way. To do that, we can call the display function on the counter. This function retrieves the current counter value and formats it either with a provided or with an automatically inferred numbering.

#set heading(numbering: "1.")

= Introduction
Some text here.

= Background
The current value is: #context {
  counter(heading).display()
}

Or in roman numerals: #context {
  counter(heading).display("I")
}

Preview
Modifying a counter

To modify a counter, you can use the step and update methods:

    The step method increases the value of the counter by one. Because counters can have multiple levels , it optionally takes a level argument. If given, the counter steps at the given depth.

    The update method allows you to arbitrarily modify the counter. In its basic form, you give it an integer (or an array for multiple levels). For more flexibility, you can instead also give it a function that receives the current value and returns a new value.

The heading counter is stepped before the heading is displayed, so Analysis gets the number seven even though the counter is at six after the second update.

#set heading(numbering: "1.")

= Introduction
#counter(heading).step()

= Background
#counter(heading).update(3)
#counter(heading).update(n => n * 2)

= Analysis
Let's skip 7.1.
#counter(heading).step(level: 2)

== Analysis
Still at #context {
  counter(heading).display()
}

Preview
Page counter

The page counter is special. It is automatically stepped at each pagebreak. But like other counters, you can also step it manually. For example, you could have Roman page numbers for your preface, then switch to Arabic page numbers for your main content and reset the page counter to one.

#set page(numbering: "(i)")

= Preface
The preface is numbered with
roman numerals.

#set page(numbering: "1 / 1")
#counter(page).update(1)

= Main text
Here, the counter is reset to one.
We also display both the current
page and total number of pages in
Arabic numbers.

Preview Preview
Custom counters

To define your own counter, call the counter function with a string as a key. This key identifies the counter globally.

#let mine = counter("mycounter")
#context mine.display() \
#mine.step()
#context mine.display() \
#mine.update(c => c * 3)
#context mine.display()

Preview
How to step

When you define and use a custom counter, in general, you should first step the counter and then display it. This way, the stepping behaviour of a counter can depend on the element it is stepped for. If you were writing a counter for, let's say, theorems, your theorem's definition would thus first include the counter step and only then display the counter and the theorem's contents.

#let c = counter("theorem")
#let theorem(it) = block[
  #c.step()
  *Theorem #context c.display():*
  #it
]

#theorem[$1 = 1$]
#theorem[$2 < 3$]

Preview

The rationale behind this is best explained on the example of the heading counter: An update to the heading counter depends on the heading's level. By stepping directly before the heading, we can correctly step from 1 to 1.1 when encountering a level 2 heading. If we were to step after the heading, we wouldn't know what to step to.

Because counters should always be stepped before the elements they count, they always start at zero. This way, they are at one for the first display (which happens after the first step).
Time travel

Counters can travel through time! You can find out the final value of the counter before it is reached and even determine what the value was at any particular location in the document.

#let mine = counter("mycounter")

= Values
#context [
  Value here: #mine.get() \
  At intro: #mine.at(<intro>) \
  Final value: #mine.final()
]

#mine.update(n => n + 3)

= Introduction <intro>
#lorem(10)

#mine.step()
#mine.step()

Preview
Other kinds of state

The counter type is closely related to state type. Read its documentation for more details on state management in Typst and why it doesn't just use normal variables for counters.
Constructor

Create a new counter identified by a key.
counter(
str
label
selector
location
function
) -> counter
key
str or label or selector or location or function
Required
Positional

The key that identifies this counter globally.

    If it is a string, creates a custom counter that is only affected by manual updates,
    If it is the page function, counts through pages,
    If it is a selector, counts through elements that match the selector. For example,
        provide an element function: counts elements of that type,
        provide a where selector: counts a type of element with specific fields,
        provide a <label>: counts elements with that label.

Definitions
get
Contextual

Retrieves the value of the counter at the current location. Always returns an array of integers, even if the counter has just one number.

This is equivalent to counter.at(here()).
self.get(
) -> intarray
display
Contextual

Displays the current value of the counter with a numbering and returns the formatted output.
self.display(
autostrfunction,both: bool,
) -> any
numbering
auto or str or function
Positional

A numbering pattern or a function, which specifies how to display the counter. If given a function, that function receives each number of the counter as a separate argument. If the amount of numbers varies, e.g. for the heading argument, you can use an argument sink.

If this is omitted or set to auto, displays the counter with the numbering style for the counted element or with the pattern "1.1" if no such style exists.

Default: auto
both
bool

If enabled, displays the current and final top-level count together. Both can be styled through a single numbering pattern. This is used by the page numbering property to display the current and total number of pages when a pattern like "1 / 1" is given.

Default: false
at
Contextual

Retrieves the value of the counter at the given location. Always returns an array of integers, even if the counter has just one number.

The selector must match exactly one element in the document. The most useful kinds of selectors for this are labels and locations.
self.at(
label
selector
location
function
) -> intarray
selector
label or selector or location or function
Required
Positional

The place at which the counter's value should be retrieved.
final
Contextual

Retrieves the value of the counter at the end of the document. Always returns an array of integers, even if the counter has just one number.
self.final(
) -> intarray
step

Increases the value of the counter by one.

The update will be in effect at the position where the returned content is inserted into the document. If you don't put the output into the document, nothing happens! This would be the case, for example, if you write let _ = counter(page).step(). Counter updates are always applied in layout order and in that case, Typst wouldn't know when to step the counter.
self.step(
level:
int
) -> content
level
int

The depth at which to step the counter. Defaults to 1.

Default: 1
update

Updates the value of the counter.

Just like with step, the update only occurs if you put the resulting content into the document.
self.update(
int
array
function
) -> content
update
int or array or function
Required
Positional

If given an integer or array of integers, sets the counter to that value. If given a function, that function receives the previous counter value (with each number as a separate argument) and has to return the new value (integer or array).



    Docs
    Reference
    Introspection
    State

state

Manages stateful parts of your document.

Let's say you have some computations in your document and want to remember the result of your last computation to use it in the next one. You might try something similar to the code below and expect it to output 10, 13, 26, and 21. However this does not work in Typst. If you test this code, you will see that Typst complains with the following error message: Variables from outside the function are read-only and cannot be modified.

// This doesn't work!
#let star = 0
#let compute(expr) = {
  star = eval(
    expr.replace("⭐", str(star))
  )
  [New value is #star.]
}

#compute("10") \
#compute("⭐ + 3") \
#compute("⭐ * 2") \
#compute("⭐ - 5")

State and document markup

Why does it do that? Because, in general, this kind of computation with side effects is problematic in document markup and Typst is upfront about that. For the results to make sense, the computation must proceed in the same order in which the results will be laid out in the document. In our simple example, that's the case, but in general it might not be.

Let's look at a slightly different, but similar kind of state: The heading numbering. We want to increase the heading counter at each heading. Easy enough, right? Just add one. Well, it's not that simple. Consider the following example:

#set heading(numbering: "1.")
#let template(body) = [
  = Outline
  ...
  #body
]

#show: template

= Introduction
...

Preview

Here, Typst first processes the body of the document after the show rule, sees the Introduction heading, then passes the resulting content to the template function and only then sees the Outline. Just counting up would number the Introduction with 1 and the Outline with 2.
Managing state in Typst

So what do we do instead? We use Typst's state management system. Calling the state function with an identifying string key and an optional initial value gives you a state value which exposes a few functions. The two most important ones are get and update:

    The get function retrieves the current value of the state. Because the value can vary over the course of the document, it is a contextual function that can only be used when context is available.

    The update function modifies the state. You can give it any value. If given a non-function value, it sets the state to that value. If given a function, that function receives the previous state and has to return the new state.

Our initial example would now look like this:

#let star = state("star", 0)
#let compute(expr) = {
  star.update(old =>
    eval(expr.replace("⭐", str(old)))
  )
  [New value is #context star.get().]
}

#compute("10") \
#compute("⭐ + 3") \
#compute("⭐ * 2") \
#compute("⭐ - 5")

Preview

State managed by Typst is always updated in layout order, not in evaluation order. The update method returns content and its effect occurs at the position where the returned content is inserted into the document.

As a result, we can now also store some of the computations in variables, but they still show the correct results:

...

#let more = [
  #compute("⭐ * 2") \
  #compute("⭐ - 5")
]

#compute("10") \
#compute("⭐ + 3") \
#more

Preview

This example is of course a bit silly, but in practice this is often exactly what you want! A good example are heading counters, which is why Typst's counting system is very similar to its state system.
Time Travel

By using Typst's state management system you also get time travel capabilities! We can find out what the value of the state will be at any position in the document from anywhere else. In particular, the at method gives us the value of the state at any particular location and the final methods gives us the value of the state at the end of the document.

...

Value at `<here>` is
#context star.at(<here>)

#compute("10") \
#compute("⭐ + 3") \
*Here.* <here> \
#compute("⭐ * 2") \
#compute("⭐ - 5")

Preview
A word of caution

To resolve the values of all states, Typst evaluates parts of your code multiple times. However, there is no guarantee that your state manipulation can actually be completely resolved.

For instance, if you generate state updates depending on the final value of a state, the results might never converge. The example below illustrates this. We initialize our state with 1 and then update it to its own final value plus 1. So it should be 2, but then its final value is 2, so it should be 3, and so on. This example displays a finite value because Typst simply gives up after a few attempts.

// This is bad!
#let x = state("key", 1)
#context x.update(x.final() + 1)
#context x.get()

Preview

In general, you should try not to generate state updates from within context expressions. If possible, try to express your updates as non-contextual values or functions that compute the new value from the previous value. Sometimes, it cannot be helped, but in those cases it is up to you to ensure that the result converges.
Constructor

Create a new state identified by a key.
state(
str,any,
) -> state
key
str
Required
Positional

The key that identifies this state.

Any updates to the state will be identified with the string key. If you construct multiple states with the same key, then updating any one will affect all of them.
init
any
Positional

The initial value of the state.

If you construct multiple states with the same key but different init values, they will each use their own initial value but share updates. Specifically, the value of a state at some location in the document will be computed from that state's initial value and all preceding updates for the state's key.

Preview

Default: none
Definitions
get
Contextual

Retrieves the value of the state at the current location.

This is equivalent to state.at(here()).
self.get(
) -> any
at
Contextual

Retrieves the value of the state at the given selector's unique match.

The selector must match exactly one element in the document. The most useful kinds of selectors for this are labels and locations.
self.at(
label
selector
location
function
) -> any
selector
label or selector or location or function
Required
Positional

The place at which the state's value should be retrieved.
final
Contextual

Retrieves the value of the state at the end of the document.
self.final(
) -> any
update

Updates the value of the state.

Returns an invisible piece of content that must be inserted into the document to take effect. This invisible content tells Typst that the specified update should take place wherever the content is inserted into the document.

State is a part of your document and runs like a thread embedded in the document content. The value of a state is the result of all state updates that happened in the document up until that point.

That's why state.update returns an invisible sliver of content that you need to return and include in the document — a state update that is not "placed" in the document does not happen, and "when" it happens is determined by where you place it. That's also why you need context to read state: You need to use the current document position to know where on the state's "thread" you are.

Storing a state update in a variable (e.g. let my-update = state("key").update(c => c * 2)) will have no effect by itself. Only once you insert the variable #my-update somewhere into the document content, the update will take effect — at the position where it was inserted. You can also use #my-update multiple times at different positions. Then, the update will take effect multiple times as well.

In contrast to get, at, and final, this function does not require context. This is because, to create the state update, we do not need to know where in the document we are. We only need this information to resolve the state's value.
self.update(any
function
) -> content
update
any or function
Required
Positional

A value to update to or a function to update with.

    If given a non-function value, sets the state to that value.
    If given a function, that function receives the state's previous value and has to return the state's new value.

When updating the state based on its previous value, you should prefer the function form instead of retrieving the previous value from the context. This allows the compiler to resolve the final state efficiently, minimizing the number of layout iterations required.

In the following example, fill.update(f => not f) will paint odd items in the bullet list as expected. However, if it's replaced with context fill.update(not fill.get()), then layout will not converge within 5 attempts, as each update will take one additional iteration to propagate.