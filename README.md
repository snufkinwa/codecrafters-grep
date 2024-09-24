<p align="center">
<img src = "https://miro.medium.com/v2/resize:fit:1222/format:webp/1*w-_7zxjx3gZgx_rLNVq60A.png">
</p>

# Build Your Own `grep` in Rust

This repository is a starting point for Rust solutions to the ["Build Your Own grep" Challenge](https://app.codecrafters.io/courses/grep/overview) from Codecrafters.

`grep` is a powerful command-line tool used to search text files for lines that match a given regular expression. It's widely used for text processing, scripting, and data analysis. In this challenge, I will be building my own `grep` implementation in Rust. Along the way, I'll explore regular expression syntax, and learn how to design parsers and lexers to evaluate them.

## Overview

The goal of this project is to build a minimal version of `grep` from scratch in Rust. This includes implementing features like:

- Searching for patterns in text files or input streams
- Supporting regular expressions for more complex pattern matching
- Handling file inputs and outputs with performance in mind

By completing this project, I will gain a deeper understanding of how text search tools like `grep` work under the hood, how regular expressions are parsed, and how to build performant command-line utilities in Rust.

## Running the Program

1. Ensure that you have `cargo` installed locally.
2. To run the program, execute:
   ```sh
   ./your_program.sh
   ```
   This will compile and run the grep implementation located in src/main.rs. The first run may be slow as it compiles the Rust project, but subsequent runs will be faster.

## Learning Outcomes

By working on this project, I'll gain hands-on experience with:

- Rust's standard library, particularly file I/O and string handling
- Regular expressions and their implementation
- Parsing and lexical analysis
- Designing efficient, scalable command-line tools

This project is an excellent opportunity to strengthen my Rust skills while learning about the foundational tools used in everyday development workflows.
