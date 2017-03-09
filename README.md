# nbted

Commandline NBT editor written in Rust. It does precisely one thing: convert NBT files to a pretty text format, and reverse the pretty text format back into NBT.

It allows you to edit NBT files with your $EDITOR (--edit or just `nbted <file>`), as well as to convert NBT files to the pretty text format (`nbted --print <file>`), and reverse them back (`nbted --reverse `). For example you might do `nbted -p file.nbt > file.txt`, edit the .txt file, and then do `nbted -r file.txt > file.nbt` to apply the edits. Do `nbted --help` for details on the options.

(Fair warning: This is a new program, while there are unit tests, and I believe the program to work, there may be unexpected edge cases. Be sure to backup files you care about before editing.)

Pretty Text Format
-----
The pretty text format is designed to be homoiconic, it precisely matches the layout of the original NBT file, with tags and values simply being converted to readable English, and indentation to make it readable being added. The only exception to this are strings, which aren't length-prefixed, but instead are quoted, and all quotes and backslashes in the string are escaped with a backslash. As an example, here is the bigtest.nbt file:
```
~/ > nbted --print bigtest.nbt
Gzip
Compound "Level"
	Long "longTest" 9223372036854775807
	Short "shortTest" 32767
	String "stringTest" "HELLO WORLD THIS IS A TEST STRING ÅÄÖ!"
	Float "floatTest" 0.49823147
	Int "intTest" 2147483647
	Compound "nested compound test"
		Compound "ham"
			String "name" "Hampus"
			Float "value" 0.75
			End
		Compound "egg"
			String "name" "Eggbert"
			Float "value" 0.5
			End
		End
	List "listTest (long)" Long 5
		11
		12
		13
		14
		15
	List "listTest (compound)" Compound 2
			String "name" "Compound tag #0"
			Long "created-on" 1264099775885
			End
			String "name" "Compound tag #1"
			Long "created-on" 1264099775885
			End
	Byte "byteTest" 127
	ByteArray "byteArrayTest (the first 1000 values of (n*n*255+n*7)%100, starting with n=0 (0, 62, 34, 16, 8, ...))" 1000
		0
		62
... 998 list elements removed to prevent the example from being too long ...
	Double "doubleTest" 0.4931287132182315
	End
End
```
The very first line in the pretty text format specifies the compression used in the NBT file, with valid values being `None`, `Gzip` and `Zlib`.

Items in compounds take the form of Type Name Value. For atomic types, the value is as one would expect, but non-atomic types are a bit more tricky. Compounds have no value. IntArrays and ByteArrays value is their length. A list's value is `Type Length`.

Compiling
-----
Compiles on stable Rust 1.14+, just with `cargo build --release`. Be sure to run the tests and rustfmt if making changes.

