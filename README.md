# downtown

Markdown to HTML converter written in Rust. Made for [askepit.github.io](https://askepit.github.io/) blog

## Usage

```bash
GENERIC USE
===========

downtown.exe -i <input> [-o <output>]

ALL PARAMETERS
==============

-i, --input      Either a path to *.md file or a path to directory which will be recursively
                 traversed and all *.md files will be processed

-o, --output     If <input> is a path to a file then <output> is a path to output *.html file. If
                 file does not exist it'll be created. If file exists it'll be overwritten. If not
                 specified, a file with same name as <input> file will be created but with .html
                 extension.
                 
                 If <input> is a path to a directory then <output> is treated as a filename to be
                 created besides each processed *.md file

-c, --config     A path to the configuration *.toml file, where you can fine-tune generator behaviour

-j, --threads    Number of threads to run. Default is 4
```

## Default conversion rules

By default, the tool generates a html-file with the following structure:

```html
<html>
<body>
Your converted markdown content
</body>
</html>
```

Markdown elements by default are converted to HTML by the following rules:

| Markdown                                                                                       | HTML                                                                                                                                              | Comments                                                                                                                                                                                       |
| ---------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Common text`                                                                                  | `<p>Common text</p>`                                                                                                                              |                                                                                                                                                                                                |
| <pre># Header<br>## Header<br>### Header<br>#### Header<br>##### Header<br>###### Header</pre> | <pre>\<h1\>Header\</h1\><br>\<h2\>Header\</h2\><br>\<h3\>Header\</h3\><br>\<h4\>Header\</h4\><br>\<h5\>Header\</h5\><br>\<h6\>Header\</h6\></pre> |                                                                                                                                                                                                |
| `*italic*` or<br>`_italic_`                                                                    | `<i>italic</i>`                                                                                                                                   | `_` in identifiers is ignored                                                                                                                                                                  |
| `**bold**` or<br>`__bold__`                                                                    | `<b>bold</b>`                                                                                                                                     | `_` in identifiers is ignored                                                                                                                                                                  |
| `***italic bold***` or<br>`___italic bold___`                                                  | `<b><i>italic bold</i></b>`                                                                                                                       | `_` in identifiers is ignored                                                                                                                                                                  |
| `~~strikethrough~~`                                                                            | `<s>strikethrough</s>`                                                                                                                            |                                                                                                                                                                                                |
| <pre>- list<br>- list</pre>                                                                    | <pre>\<ul><br>  \<li>list\</li><br>  \<li>list\</li><br>\</ul><br></pre>                                                                          | Nesting is *not supported yet*. Numeric lists *are not supported yet *                                                                                                                         |
| <pre>\`console.log("nice!")\`</pre>                                                            | <pre>\<code>console.log("nice!")\</code></pre>                                                                                                    |                                                                                                                                                                                                |
| <pre>\`\`\`cpp<br>int x;<br>x = 15;<br>\`\`\`</pre>                                            | <pre>\<pre>\<code class=\"language-cpp\"><br>int x;<br>x = 15;<br>\</code>\</pre><br></pre>                                                       |                                                                                                                                                                                                |
| `[Link caption](https://link-url.com)`                                                         | <pre>\<a href=\"https://link-url.com">Link caption\</a><br></pre>                                                                                 |                                                                                                                                                                                                |
| `![Image caption](https://image-url.jpg)`                                                      | <pre>\<img src=\"https://image-url.jpg\" alt=\"Image caption\"><br></pre>                                                                         |                                                                                                                                                                                                |
| <pre>\$\$<br>y = sin(x)<br>$$</pre>                                                            | <pre>\$\$<br>y = sin(x)<br>$$</pre>                                                                                                               | Copies as it is to allow external latex libraries to process LaTeX text                                                                                                                        |
| <pre>> text<br>> text</pre>                                                                    | <pre>\<blockquote><br>  \<p>text\</p><br>  \<p>text\</p><br>\</blockquote></pre>                                                                  |                                                                                                                                                                                                |
| `---`                                                                                          | <pre>\<hr></pre>                                                                                                                                  |                                                                                                                                                                                                |
| `![[some obsidian local article]]`                                                             | <pre>\<div class="parse-error">some obsidian local article\</div></pre>                                                                           | Any stuff that could not be parsed properly converts to a "parse-error" which you can detect then in your html-page if you properly prepare alarming css formatting for the class .parse-error |

## Generator configuration

You can customize generation behaviour by specifying a `-c` flag with a path to a special configuration file. Read about it [here](config/CONFIG.md).

## Plans

- Nested lists
- Numeric lists
- Tables

## How many threads should I use?

If you do not specify `-j` option it will use 4 threads by default which is okay for the most cases. Bear in mind that bigger amount of threads is not always better. Tests on my PC with `Processor 12th Gen Intel(R) Core(TM) i5-12600K, 3700 Mhz, 10 Core(s), 16 Logical Processor(s)` showed that sweet pot is located somewhere between 4-7 threads and using 16 on the other hand degrade speed back to almost a single-threaded scenario:

![](https://habrastorage.org/webt/lu/rz/4z/lurz4z4aqxmrr9tn9ds-swnpxbw.png)
