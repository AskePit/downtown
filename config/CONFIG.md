Configuration file is written in [TOML](https://toml.io/en/) format. It consists of two sections: `[page]` and `[tags]`.

## [page] section

It specifies what should be appended to the main html-body at the beginning and at the end of the main html content. Variables `prologue` and `epilogue` are responsible for that:

```toml
[page]
prologue = '''
<!DOCTYPE html>
<html>
<head lang="en">
    <meta charset="UTF-8">
    <title>Test page</title>
</head>
<body>
'''

epilogue = '''
</body>
</html>
'''
```

In the example above, if generator created for us a text like `<html><body><p>Hello, World!</p></body></html>` then we'll get a html page which content looks like:

```html
<!DOCTYPE html>
<html>
<head lang="en">
    <meta charset="UTF-8">
    <title>Test page</title>
</head>
<body>
<p>Hello, World!</p>
</body>
</html>
```

---

Remember that by default generator generates `<html><body>{text}</body></html>` content, so if you don't use configuration file, or it has empty `prologue` and `epilogue` variables then you'll get:

```html
<html>
<body>
<p>Hello, World!</p>
</body>
</html>
```

If `prologue` variable is specified then default `<html><body>` will be replaced with `prologue`s content. If `epilogue` variable is specified then default `</body></html>` will be replaced with `epilogue`s content.

## [tags] section

The section has a dozen of variables to fine-tune almost every element of generation:

| Variable          | Parameters             | Default value                                            |
| ----------------- | ---------------------- | -------------------------------------------------------- |
| `image`           | `{src}`<br>`{caption}` | `<img src="{src}" alt="{caption}">`                      |
| `link`            | `{src}`<br>`{caption}` | `<a href="{src}">{caption}</a>`                          |
| `latex`           | `{text}`               | `<p class="latex">{text}</p>`                            |
| `code`            | `{lang}`<br>`{text}`   | `<pre><code class="language-{lang}">{text}</code></pre>` |
| `code-inline`     | `{text}`               | `<code>{text}</code>`                                    |
| `blockquote`      | `{text}`               | `<blockquote>{text}</blockquote>`                        |
| `horizontal_line` |                        | `<hr>`                                                   |
| `paragraph`       | `{text}`               | `<p>{text}</p>`                                          |
| `bold`            | `{text}`               | `<b>{text}</b>`                                          |
| `italic`          | `{text}`               | `<i>{text}</i>`                                          |
| `strikethrough`   | `{text}`               | `<s>{text}</s>`                                          |
| `header`          | `{level}`<br>`{text}`  | `<h{level}>{text}</h{level}>`                            |
| `error`           | `{text}`               | `<div class="parse-error">{text}</div>`                  |

## Examples

To see configuration examples you can look at repo files:

- `config/default_config_sample.toml`. The file shows how to configure the behaviour that the tool uses by default internally
- `config/custom_config_sample.toml`. The file shows a custom configuration where generation result uses exotic tags and different `<div>` wrappers around content

## Usage

If you have a configuration file at path `config/configuration.toml` then you can execute the tool in a following manner:

```bash
./downtown.exe -i input.md -o index.html -c config/configuration.toml
```