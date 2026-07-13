/*
 | 标题   | `#` `##` `###` |    <hx>
 | 段落   | 普通文本           | <p>
 | 粗体   | `**text**`     |  <strong>
 | 行内代码 | `` `code` ``   | <code>
 | 代码块  | ` ``` `        | <pre><code>
 | 无序列表 | `-`            | <ul>
 | 有序列表 | `1.`           | <li>
 | 引用   | `>`            | <blockquote>
 | 链接   | `[text](url)`  | <a>
 | 分割线  | `---`          |     <hr>
 | 斜体  | `*text*`   | <em>
 | 图片  | `![]()`    | <img>
 | 删除线 | `~~text~~` | <del>
 | 表格  | `\|`       | <table><tr><td>
*/

const css : &'static str = r#"
/* ===========================
   Base
   =========================== */

body {
    max-width: 900px;
    margin: 40px auto;
    padding: 0 24px;
    font-family:
        -apple-system,
        BlinkMacSystemFont,
        "Segoe UI",
        Helvetica,
        Arial,
        sans-serif;
    font-size: 16px;
    line-height: 1.8;
    color: #24292f;
    background: #ffffff;
}

/* ===========================
   Heading
   =========================== */

h1,
h2,
h3,
h4,
h5,
h6 {
    margin-top: 1.8em;
    margin-bottom: 0.8em;
    font-weight: 600;
    line-height: 1.25;
}

h1,
h2 {
    border-bottom: 1px solid #d8dee4;
    padding-bottom: .3em;
}

h1 {
    font-size: 2em;
}

h2 {
    font-size: 1.6em;
}

h3 {
    font-size: 1.35em;
}

h4 {
    font-size: 1.15em;
}

h5 {
    font-size: 1em;
}

h6 {
    color: #656d76;
}

/* ===========================
   Paragraph
   =========================== */

p {
    margin: 16px 0;
}

/* ===========================
   Link
   =========================== */

a {
    color: #0969da;
    text-decoration: none;
}

a:hover {
    text-decoration: underline;
}

/* ===========================
   Strong / Emphasis
   =========================== */

strong {
    font-weight: 700;
}

em {
    font-style: italic;
}

del {
    color: #656d76;
}

/* ===========================
   Inline Code
   =========================== */

code {
    padding: .15em .4em;
    border-radius: 6px;
    background: #f6f8fa;
    color: #d73a49;
    font-family:
        SFMono-Regular,
        Consolas,
        monospace;
    font-size: .9em;
}

/* ===========================
   Code Block
   =========================== */

pre {
    padding: 16px;
    overflow: auto;
    border-radius: 10px;
    background: #282c34;
    color: #abb2bf;
}

pre code {
    padding: 0;
    background: transparent;
    color: inherit;
    font-size: 14px;
}

/* ===========================
   Quote
   =========================== */

blockquote {
    margin: 20px 0;
    padding: 8px 16px;
    border-left: 4px solid #4f8ef7;
    background: #f6f8fa;
    color: #57606a;
}

blockquote p:first-child {
    margin-top: 0;
}

blockquote p:last-child {
    margin-bottom: 0;
}

/* ===========================
   List
   =========================== */

ul,
ol {
    padding-left: 2em;
}

li {
    margin: .4em 0;
}

/* ===========================
   Horizontal Rule
   =========================== */

hr {
    margin: 32px 0;
    border: none;
    border-top: 1px solid #d8dee4;
}

/* ===========================
   Table
   =========================== */

table {
    width: 100%;
    border-collapse: collapse;
    margin: 20px 0;
}

thead {
    background: #f6f8fa;
}

th,
td {
    padding: 10px 14px;
    border: 1px solid #d0d7de;
}

th {
    font-weight: 600;
}

tbody tr:nth-child(even) {
    background: #fafbfc;
}

/* ===========================
   Image
   =========================== */

img {
    max-width: 100%;
    border-radius: 10px;
    box-shadow: 0 3px 12px rgba(0,0,0,.15);
}

/* ===========================
   Selection
   =========================== */

::selection {
    background: #b6d4ff;
}

/* ===========================
   Scrollbar（可选）
   =========================== */

::-webkit-scrollbar {
    width: 10px;
    height: 10px;
}

::-webkit-scrollbar-thumb {
    background: #c1c1c1;
    border-radius: 5px;
}

::-webkit-scrollbar-thumb:hover {
    background: #999;
}
"#;

pub fn generate_html(input: &str) -> String {
    String::new()
}