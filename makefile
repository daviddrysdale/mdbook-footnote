BINARY=${HOME}/.cargo/bin/mdbook-footnote

all: sample/book1/book/chapter_1.html sample/book2/book/chapter_1.html

install: $(BINARY)
$(BINARY): src/main.rs
	cargo install --offline --path .

sample/book1/book/chapter_1.html: sample/book1/src/chapter_1.md $(BINARY)
	(cd sample/book1 && mdbook build)
sample/book2/book/chapter_1.html: sample/book2/src/chapter_1.md $(BINARY)
	(cd sample/book2 && mdbook build)

compare: compare1 compare2
compare1: sample/book1/book/chapter_1.html
	echo "Comparing HTML output with expected for book1"
	diff sample/expected/book1_chap1.html sample/book1/book/chapter_1.html
compare2: sample/book2/book/chapter_1.html
	echo "Comparing HTML output with expected for book2"
	diff sample/expected/book2_chap1.html sample/book2/book/chapter_1.html

clean:
	rm -rf sample/book1/book
	rm -rf sample/book2/book

# Target that installs the version of mdbook used to create the sample/expected/ files
dependencies:
	cargo install mdbook@=0.5.1
