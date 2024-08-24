# downtown

Markdown to HTML converter written in Rust. Made for askepit.github.io blog

## Usage

```bash
downtown.exe -i <input> [-j <number>] [-o <output>]

-i, --input      Input *.md file
-o, --output     Output *.html file. If not specified, a file with same name as <input> file will be
                 created but with .html extension
-j, --threads    Number of threads to run. Default is 4"#
```

The tool generates a html-file **without \<head\> and \<body\>**, but only content which you normally see inside a \<body\>. Nevertheless, it could be opened and viewed by browsers as it is. 

## How many threads should I use?

If you do not specify `-j` option it will use 4 threads by default. Bear in mind that bigger amount of threads is not always better. Tests on my PC with `Processor	12th Gen Intel(R) Core(TM) i5-12600K, 3700 Mhz, 10 Core(s), 16 Logical Processor(s)` showed that sweet pot is located somewhere between 4-6 threads and using 16 on the other hand degrade speed back to almost a single-threaded scenario:

![](https://habrastorage.org/webt/lu/rz/4z/lurz4z4aqxmrr9tn9ds-swnpxbw.png)
