# pint 🍺 
### A [(P)iet](https://www.dangermouse.net/esoteric/piet.html) (Int)erpreter with a builtin png-decoder.<br>
Piet is a programming language in which programs look like abstract paintings. The language is named after Piet Mondrian, who pioneered the field of geometric abstract art.

<img width=200 align="left" src="./tests/fixtures/piet_hello_world.png">
<h4 align="left">This is a piet program that prints "Hello World!"</h4>

## Installation
```
cargo install pint
```
or you can clone this repo and build it from source

## Usage
```
pint some_test.png
```
The [codel-size](http://www.majcher.com/code/piet/Piet-Interpreter.html#codels) is inferred automatically.
You can also pass it manually:
```
pint some_test.png -c <codel_size>
```
Since the png-decoder is built from scratch it only implements the most common [Png color-types](https://www.w3.org/TR/PNG/#6Colour-values) TruecolorRGB and Indexed<br>
There is currently no support for gifs

## Tests
Typing `make` shows you the options for this crate.
#### unit-tests
```
$ make unit_test
or
$ cargo t
```
There also integration tests that check the result of the test-images located in tests/fixtures
```
$ make integration_tests
or
$ bash tests/integration_tests
```

## Contribution
I would appreciate any kind of contribution (as long as the tests pass :)) or feedback since this is my first time writing rust. Maybe if there is somebody willing to finish the png decoder to support all kinds of pngs
