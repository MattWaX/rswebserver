build-html:
	pandoc -s md/index.md -o www/index.html --toc --css style.css --metadata title="Web Server asincrono scritto in rust"

build: build-html
