
# Gill ![Image](logo/gilllogo2.png)

Gill is a programming language similar to the Lisp, C, and Rust that would be mainly used to make software and maybe even an operating system. 
Gill is open source. So that means everyone can make their own programming language.

Examples:
Introduce program name (optional) and print:
```
NAME="printtoconsole"
DESC="Print program"

func main() {
	print.console("Hello");
}
```
Print random line:
```
NAME="randomprint"

local messages=("Hi", "Hello");

func main() {
	print.console.random(messages);
}
```
