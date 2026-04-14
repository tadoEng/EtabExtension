
    Docs
    Reference
    Foundations
    Array

array

A sequence of values.

You can construct an array by enclosing a comma-separated sequence of values in parentheses. The values do not have to be of the same type.

You can access and update array items with the .at() method. Indices are zero-based and negative indices wrap around to the end of the array. You can iterate over an array using a for loop. Arrays can be added together with the + operator, joined together and multiplied with integers.

Note: An array of length one needs a trailing comma, as in (1,). This is to disambiguate from a simple parenthesized expressions like (1 + 2) * 3. An empty array is written as ().
Example

#let values = (1, 7, 4, -3, 2)

#values.at(0) \
#(values.at(0) = 3)
#values.at(-1) \
#values.find(calc.even) \
#values.filter(calc.odd) \
#values.map(calc.abs) \
#values.rev() \
#(1, (2, 3)).flatten() \
#(("A", "B", "C")
    .join(", ", last: " and "))

Preview
Constructor

Converts a value to an array.

Note that this function is only intended for conversion of a collection-like value to an array, not for creation of an array from individual items. Use the array syntax (1, 2, 3) (or (1,) for a single-element array) instead.

#let hi = "Hello 😃"
#array(bytes(hi))

Preview
array(
bytes
array
version
) -> array
value
bytes or array or version
Required
Positional

The value that should be converted to an array.
Definitions
len

The number of values in the array.
self.len(
) -> int
first

Returns the first item in the array. May be used on the left-hand side an assignment. Returns the default value if the array is empty or fails with an error is no default value was specified.
self.first(
default:
any
) -> any
default
any

A default value to return if the array is empty.
last

Returns the last item in the array. May be used on the left-hand side of an assignment. Returns the default value if the array is empty or fails with an error is no default value was specified.
self.last(
default:
any
) -> any
default
any

A default value to return if the array is empty.
at

Returns the item at the specified index in the array. May be used on the left-hand side of an assignment. Returns the default value if the index is out of bounds or fails with an error if no default value was specified.
self.at(
int,default: any,
) -> any
index
int
Required
Positional

The index at which to retrieve the item. If negative, indexes from the back.
default
any

A default value to return if the index is out of bounds.
push

Adds a value to the end of the array.
self.push(any
)
value
any
Required
Positional

The value to insert at the end of the array.
pop

Removes the last item from the array and returns it. Fails with an error if the array is empty.
self.pop(
) -> any
insert

Inserts a value into the array at the specified index, shifting all subsequent elements to the right. Fails with an error if the index is out of bounds.

To replace an element of an array, use at.
self.insert(
int,any,
)
index
int
Required
Positional

The index at which to insert the item. If negative, indexes from the back.
value
any
Required
Positional

The value to insert into the array.
remove

Removes the value at the specified index from the array and return it.
self.remove(
int,default: any,
) -> any
index
int
Required
Positional

The index at which to remove the item. If negative, indexes from the back.
default
any

A default value to return if the index is out of bounds.
slice

Extracts a subslice of the array. Fails with an error if the start or end index is out of bounds.
self.slice(
int,noneint,count: int,
) -> array
start
int
Required
Positional

The start index (inclusive). If negative, indexes from the back.
end
none or int
Positional

The end index (exclusive). If omitted, the whole slice until the end of the array is extracted. If negative, indexes from the back.

Default: none
count
int

The number of items to extract. This is equivalent to passing start + count as the end position. Mutually exclusive with end.
contains

Whether the array contains the specified value.

This method also has dedicated syntax: You can write 2 in (1, 2, 3) instead of (1, 2, 3).contains(2).
self.contains(any
) -> bool
value
any
Required
Positional

The value to search for.
find

Searches for an item for which the given function returns true and returns the first match or none if there is no match.
self.find(
function
) -> anynone
searcher
function
Required
Positional

The function to apply to each item. Must return a boolean.
position

Searches for an item for which the given function returns true and returns the index of the first match or none if there is no match.
self.position(
function
) -> noneint
searcher
function
Required
Positional

The function to apply to each item. Must return a boolean.
range

Create an array consisting of a sequence of numbers.

If you pass just one positional parameter, it is interpreted as the end of the range. If you pass two, they describe the start and end of the range.

This function is available both in the array function's scope and globally.

#range(5) \
#range(2, 5) \
#range(20, step: 4) \
#range(21, step: 4) \
#range(5, 2, step: -1)

Preview
array.range(
int,int,step: int,
) -> array
start
int
Positional

The start of the range (inclusive).

Default: 0
end
int
Required
Positional

The end of the range (exclusive).
step
int

The distance between the generated numbers.

Default: 1
filter

Produces a new array with only the items from the original one for which the given function returns true.
self.filter(
function
) -> array
test
function
Required
Positional

The function to apply to each item. Must return a boolean.
map

Produces a new array in which all items from the original one were transformed with the given function.
self.map(
function
) -> array
mapper
function
Required
Positional

The function to apply to each item.
enumerate

Returns a new array with the values alongside their indices.

The returned array consists of (index, value) pairs in the form of length-2 arrays. These can be destructured with a let binding or for loop.

#for (i, value) in ("A", "B", "C").enumerate() {
  [#i: #value \ ]
}

#("A", "B", "C").enumerate(start: 1)

Preview
self.enumerate(
start:
int
) -> array
start
int

The index returned for the first pair of the returned list.

Default: 0
zip

Zips the array with other arrays.

Returns an array of arrays, where the ith inner array contains all the ith elements from each original array.

If the arrays to be zipped have different lengths, they are zipped up to the last element of the shortest array and all remaining elements are ignored.

This function is variadic, meaning that you can zip multiple arrays together at once: (1, 2).zip(("A", "B"), (10, 20)) yields ((1, "A", 10), (2, "B", 20)).
self.zip(
exact: bool,..array,
) -> array
exact
bool

Whether all arrays have to have the same length. For example, (1, 2).zip((1, 2, 3), exact: true) produces an error.

Default: false
others
array
Required
Positional
Variadic

The arrays to zip with.
fold

Folds all items into a single value using an accumulator function.

#let array = (1, 2, 3, 4)
#array.fold(0, (acc, x) => acc + x)

Preview
self.fold(
any,function,
) -> any
init
any
Required
Positional

The initial value to start with.
folder
function
Required
Positional

The folding function. Must have two parameters: One for the accumulated value and one for an item.
sum

Sums all items (works for all types that can be added).
self.sum(
default:
any
) -> any
default
any

What to return if the array is empty. Must be set if the array can be empty.
product

Calculates the product of all items (works for all types that can be multiplied).
self.product(
default:
any
) -> any
default
any

What to return if the array is empty. Must be set if the array can be empty.
any

Whether the given function returns true for any item in the array.
self.any(
function
) -> bool
test
function
Required
Positional

The function to apply to each item. Must return a boolean.
all

Whether the given function returns true for all items in the array.
self.all(
function
) -> bool
test
function
Required
Positional

The function to apply to each item. Must return a boolean.
flatten

Combine all nested arrays into a single flat one.
self.flatten(
) -> array
rev

Return a new array with the same items, but in reverse order.
self.rev(
) -> array
split

Split the array at occurrences of the specified value.

#(1, 1, 2, 3, 2, 4, 5).split(2)

Preview
self.split(any
) -> array
at
any
Required
Positional

The value to split at.
join

Combine all items in the array into one.
self.join(
anynone,last: any,default: anynone,
) -> any
separator
any or none
Positional

A value to insert between each item of the array.

Default: none
last
any

An alternative separator between the last two items.
default
any or none

What to return if the array is empty.

Default: none
intersperse

Returns an array with a copy of the separator value placed between adjacent elements.

#("A", "B", "C").intersperse("-")

Preview
self.intersperse(any
) -> array
separator
any
Required
Positional

The value that will be placed between each adjacent element.
chunks

Splits an array into non-overlapping chunks, starting at the beginning, ending with a single remainder chunk.

All chunks but the last have chunk-size elements. If exact is set to true, the remainder is dropped if it contains less than chunk-size elements.

#let array = (1, 2, 3, 4, 5, 6, 7, 8)
#array.chunks(3) \
#array.chunks(3, exact: true)

Preview
self.chunks(
int,exact: bool,
) -> array
chunk-size
int
Required
Positional

How many elements each chunk may at most contain.
exact
bool

Whether to keep the remainder if its size is less than chunk-size.

Default: false
windows

Returns sliding windows of window-size elements over an array.

If the array length is less than window-size, this will return an empty array.

#let array = (1, 2, 3, 4, 5, 6, 7, 8)
#array.windows(5)

Preview
self.windows(
int
) -> array
window-size
int
Required
Positional

How many elements each window will contain.
sorted

Return a sorted version of this array, optionally by a given key function. The sorting algorithm used is stable.

Returns an error if a pair of values selected for comparison could not be compared, or if the key or comparison function (if given) yield an error.

To sort according to multiple criteria at once, e.g. in case of equality between some criteria, the key function can return an array. The results are in lexicographic order.

#let array = (
  (a: 2, b: 4),
  (a: 1, b: 5),
  (a: 2, b: 3),
)
#array.sorted(key: it => (it.a, it.b))

Preview
self.sorted(
key: function,by: function,
) -> array
key
function

If given, applies this function to each element in the array to determine the keys to sort by.
by
function

If given, uses this function to compare every two elements in the array.

The function will receive two elements in the array for comparison, and should return a boolean indicating their order: true indicates that the elements are in order, while false indicates that they should be swapped. To keep the sort stable, if the two elements are equal, the function should return true.

If this function does not order the elements properly (e.g., by returning false for both (x, y) and (y, x), or for (x, x)), the resulting array will be in unspecified order.

When used together with key, by will be passed the keys instead of the elements.

Preview
dedup

Deduplicates all items in the array.

Returns a new array with all duplicate items removed. Only the first element of each duplicate is kept.

#(3, 3, 1, 2, 3).dedup()

Preview
self.dedup(
key:
function
) -> array
key
function

If given, applies this function to each element in the array to determine the keys to deduplicate by.

Preview
to-dict

Converts an array of pairs into a dictionary. The first value of each pair is the key, the second the value.

If the same key occurs multiple times, the last value is selected.

#(
  ("apples", 2),
  ("peaches", 3),
  ("apples", 5),
).to-dict()

Preview
self.to-dict(
) -> dictionary
reduce

Reduces the elements to a single one, by repeatedly applying a reducing operation.

If the array is empty, returns none, otherwise, returns the result of the reduction.

The reducing function is a closure with two arguments: an "accumulator", and an element.

For arrays with at least one element, this is the same as array.fold with the first element of the array as the initial accumulator value, folding every subsequent element into it.

#let array = (2, 1, 4, 3)
#array.reduce((acc, x) => calc.max(acc, x))

Preview
self.reduce(
function
) -> any
reducer
function
Required
Positional

The reducing function. Must have two parameters: One for the accumulated value and one for an item.



    Docs
    Reference
    Foundations
    Dictionary

dictionary

A map from string keys to values.

You can construct a dictionary by enclosing comma-separated key: value pairs in parentheses. The values do not have to be of the same type. Since empty parentheses already yield an empty array, you have to use the special (:) syntax to create an empty dictionary.

A dictionary is conceptually similar to an array, but it is indexed by strings instead of integers. You can access and create dictionary entries with the .at() method. If you know the key statically, you can alternatively use field access notation (.key) to access the value. To check whether a key is present in the dictionary, use the in keyword.

You can iterate over the pairs in a dictionary using a for loop. This will iterate in the order the pairs were inserted / declared initially.

Dictionaries can be added with the + operator and joined together. They can also be spread into a function call or another dictionary1 with the ..spread operator. In each case, if a key appears multiple times, the last value will override the others.
Example

#let dict = (
  name: "Typst",
  born: 2019,
)

#dict.name \
#(dict.launch = 20)
#dict.len() \
#dict.keys() \
#dict.values() \
#dict.at("born") \
#dict.insert("city", "Berlin")
#("name" in dict)

Preview
1

When spreading into a dictionary, if all items between the parentheses are spread, you have to use the special (:..spread) syntax. Otherwise, it will spread into an array.
Constructor

Converts a value into a dictionary.

Note that this function is only intended for conversion of a dictionary-like value to a dictionary, not for creation of a dictionary from individual pairs. Use the dictionary syntax (key: value) instead.

#dictionary(sys).at("version")

Preview
dictionary(
module
) -> dictionary
value
module
Required
Positional

The value that should be converted to a dictionary.
Definitions
len

The number of pairs in the dictionary.
self.len(
) -> int
at

Returns the value associated with the specified key in the dictionary. May be used on the left-hand side of an assignment if the key is already present in the dictionary. Returns the default value if the key is not part of the dictionary or fails with an error if no default value was specified.
self.at(
str,default: any,
) -> any
key
str
Required
Positional

The key at which to retrieve the item.
default
any

A default value to return if the key is not part of the dictionary.
insert

Inserts a new pair into the dictionary. If the dictionary already contains this key, the value is updated.

To insert multiple pairs at once, you can just alternatively another dictionary with the += operator.
self.insert(
str,any,
)
key
str
Required
Positional

The key of the pair that should be inserted.
value
any
Required
Positional

The value of the pair that should be inserted.
remove

Removes a pair from the dictionary by key and return the value.
self.remove(
str,default: any,
) -> any
key
str
Required
Positional

The key of the pair to remove.
default
any

A default value to return if the key does not exist.
keys

Returns the keys of the dictionary as an array in insertion order.
self.keys(
) -> array
values

Returns the values of the dictionary as an array in insertion order.
self.values(
) -> array
pairs

Returns the keys and values of the dictionary as an array of pairs. Each pair is represented as an array of length two.
self.pairs(
) -> array



    Docs
    Reference
    Foundations
    String

str

A sequence of Unicode codepoints.

You can iterate over the grapheme clusters of the string using a for loop. Grapheme clusters are basically characters but keep together things that belong together, e.g. multiple codepoints that together form a flag emoji. Strings can be added with the + operator, joined together and multiplied with integers.

Typst provides utility methods for string manipulation. Many of these methods (e.g., split, trim and replace) operate on patterns: A pattern can be either a string or a regular expression. This makes the methods quite versatile.

All lengths and indices are expressed in terms of UTF-8 bytes. Indices are zero-based and negative indices wrap around to the end of the string.

You can convert a value to a string with this type's constructor.
Example

#"hello world!" \
#"\"hello\n  world\"!" \
#"1 2 3".split() \
#"1,2;3".split(regex("[,;]")) \
#(regex("\\d+") in "ten euros") \
#(regex("\\d+") in "10 euros")

Preview
Escape sequences

Just like in markup, you can escape a few symbols in strings:

    \\ for a backslash
    \" for a quote
    \n for a newline
    \r for a carriage return
    \t for a tab
    \u{1f600} for a hexadecimal Unicode escape sequence

Constructor

Converts a value to a string.

    Integers are formatted in base 10. This can be overridden with the optional base parameter.
    Floats are formatted in base 10 and never in exponential notation.
    Negative integers and floats are formatted with the Unicode minus sign ("−" U+2212) instead of the ASCII minus sign ("-" U+002D).
    From labels the name is extracted.
    Bytes are decoded as UTF-8.

If you wish to convert from and to Unicode code points, see the to-unicode and from-unicode functions.

#str(10) \
#str(4000, base: 16) \
#str(2.7) \
#str(1e8) \
#str(<intro>)

Preview
str(
intfloatstrbyteslabeldecimalversiontype,base: int,
) -> str
value
int or float or str or bytes or label or decimal or version or type
Required
Positional

The value that should be converted to a string.
base
int

The base (radix) to display integers in, between 2 and 36.

Default: 10
Definitions
len

The length of the string in UTF-8 encoded bytes.
self.len(
) -> int
first

Extracts the first grapheme cluster of the string.

Returns the provided default value if the string is empty or fails with an error if no default value was specified.
self.first(
default:
str
) -> str
default
str

A default value to return if the string is empty.
last

Extracts the last grapheme cluster of the string.

Returns the provided default value if the string is empty or fails with an error if no default value was specified.
self.last(
default:
str
) -> str
default
str

A default value to return if the string is empty.
at

Extracts the first grapheme cluster after the specified index. Returns the default value if the index is out of bounds or fails with an error if no default value was specified.
self.at(
int,default: any,
) -> any
index
int
Required
Positional

The byte index. If negative, indexes from the back.
default
any

A default value to return if the index is out of bounds.
slice

Extracts a substring of the string. Fails with an error if the start or end index is out of bounds.
self.slice(
int,noneint,count: int,
) -> str
start
int
Required
Positional

The start byte index (inclusive). If negative, indexes from the back.
end
none or int
Positional

The end byte index (exclusive). If omitted, the whole slice until the end of the string is extracted. If negative, indexes from the back.

Default: none
count
int

The number of bytes to extract. This is equivalent to passing start + count as the end position. Mutually exclusive with end.
clusters

Returns the grapheme clusters of the string as an array of substrings.
self.clusters(
) -> array
codepoints

Returns the Unicode codepoints of the string as an array of substrings.
self.codepoints(
) -> array
to-unicode

Converts a character into its corresponding code point.

#"a".to-unicode() \
#("a\u{0300}"
   .codepoints()
   .map(str.to-unicode))

Preview
str.to-unicode(
str
) -> int
character
str
Required
Positional

The character that should be converted.
from-unicode

Converts a unicode code point into its corresponding string.

#str.from-unicode(97)

Preview
str.from-unicode(
int
) -> str
value
int
Required
Positional

The code point that should be converted.
normalize

Normalizes the string to the given Unicode normal form.

This is useful when manipulating strings containing Unicode combining characters.

#assert.eq("é".normalize(form: "nfd"), "e\u{0301}")
#assert.eq("ſ́".normalize(form: "nfkc"), "ś")

self.normalize(
form:
str
) -> str
form
str
Variant	Details
"nfc"	

Canonical composition where e.g. accented letters are turned into a single Unicode codepoint.
"nfd"	

Canonical decomposition where e.g. accented letters are split into a separate base and diacritic.
"nfkc"	

Like NFC, but using the Unicode compatibility decompositions.
"nfkd"	

Like NFD, but using the Unicode compatibility decompositions.

Default: "nfc"
contains

Whether the string contains the specified pattern.

This method also has dedicated syntax: You can write "bc" in "abcd" instead of "abcd".contains("bc").
self.contains(
str
regex
) -> bool
pattern
str or regex
Required
Positional

The pattern to search for.
starts-with

Whether the string starts with the specified pattern.
self.starts-with(
str
regex
) -> bool
pattern
str or regex
Required
Positional

The pattern the string might start with.
ends-with

Whether the string ends with the specified pattern.
self.ends-with(
str
regex
) -> bool
pattern
str or regex
Required
Positional

The pattern the string might end with.
find

Searches for the specified pattern in the string and returns the first match as a string or none if there is no match.
self.find(
str
regex
) -> nonestr
pattern
str or regex
Required
Positional

The pattern to search for.
position

Searches for the specified pattern in the string and returns the index of the first match as an integer or none if there is no match.
self.position(
str
regex
) -> noneint
pattern
str or regex
Required
Positional

The pattern to search for.
match

Searches for the specified pattern in the string and returns a dictionary with details about the first match or none if there is no match.

The returned dictionary has the following keys:

    start: The start offset of the match
    end: The end offset of the match
    text: The text that matched.
    captures: An array containing a string for each matched capturing group. The first item of the array contains the first matched capturing, not the whole match! This is empty unless the pattern was a regex with capturing groups.

#let pat = regex("not (a|an) (apple|cat)")
#"I'm a doctor, not an apple.".match(pat) \
#"I am not a cat!".match(pat)

Preview

Preview
self.match(
str
regex
) -> nonedictionary
pattern
str or regex
Required
Positional

The pattern to search for.
matches

Searches for the specified pattern in the string and returns an array of dictionaries with details about all matches. For details about the returned dictionaries, see above.

#"Day by Day.".matches("Day")

Preview
self.matches(
str
regex
) -> array
pattern
str or regex
Required
Positional

The pattern to search for.
replace

Replace at most count occurrences of the given pattern with a replacement string or function (beginning from the start). If no count is given, all occurrences are replaced.
self.replace(
strregex,strfunction,count: int,
) -> str
pattern
str or regex
Required
Positional

The pattern to search for.
replacement
str or function
Required
Positional

The string to replace the matches with or a function that gets a dictionary for each match and can return individual replacement strings.

The dictionary passed to the function has the same shape as the dictionary returned by match.
count
int

If given, only the first count matches of the pattern are placed.
trim

Removes matches of a pattern from one or both sides of the string, once or repeatedly and returns the resulting string.
self.trim(
nonestrregex,at: alignment,repeat: bool,
) -> str
pattern
none or str or regex
Positional

The pattern to search for. If none, trims white spaces.

Default: none
at
alignment

Can be start or end to only trim the start or end of the string. If omitted, both sides are trimmed.
repeat
bool

Whether to repeatedly removes matches of the pattern or just once. Defaults to true.

Default: true
split

Splits a string at matches of a specified pattern and returns an array of the resulting parts.

When the empty string is used as a separator, it separates every character (i.e., Unicode code point) in the string, along with the beginning and end of the string. In practice, this means that the resulting list of parts will contain the empty string at the start and end of the list.
self.split(
none
str
regex
) -> array
pattern
none or str or regex
Positional

The pattern to split at. Defaults to whitespace.

Default: none
rev

Reverse the string.
self.rev(
) -> str


    Docs
    Reference
    Foundations
    Evaluate

eval

Evaluates a string as Typst code.

This function should only be used as a last resort.
Example

#eval("1 + 1") \
#eval("(1, 2, 3, 4)").len() \
#eval("*Markup!*", mode: "markup") \

Preview
Parameters
eval(
str,mode: str,scope: dictionary,
) -> any
source
str
Required
Positional

A string of Typst code to evaluate.
mode
str

The syntactical mode in which the string is parsed.

Preview
Variant	Details
"markup"	

Evaluate as markup, as in a Typst file.
"math"	

Evaluate as math, as in an equation.
"code"	

Evaluate as code, as after a hash.

Default: "code"
scope
dictionary

A scope of definitions that are made available.

Preview

Default: (:) 