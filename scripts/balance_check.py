#!/usr/bin/env python3
"""String/char/comment-aware brace-balance checker for the .rs files in this
crate. The naive 'count every { in the file' approach false-positives on any
string literal containing braces (JSON fragments, format strings), so this
walks the file respecting string/char literals and // and /* */ comments."""
import sys, glob

def check(path):
    s = open(path, encoding='utf-8').read()
    i, n = 0, len(s)
    pairs = {"}": "{", ")": "(", "]": "["}
    stack = []
    line = 1
    in_str = in_char = in_line_comment = in_block_comment = False
    problems = []
    while i < n:
        c = s[i]
        if c == '\n':
            line += 1
            in_line_comment = False
        if in_line_comment:
            i += 1; continue
        if in_block_comment:
            if s[i:i+2] == '*/':
                in_block_comment = False; i += 2; continue
            i += 1; continue
        if in_str:
            if c == '\\': i += 2; continue
            if c == '"': in_str = False
            i += 1; continue
        if in_char:
            if c == '\\': i += 2; continue
            if c == "'": in_char = False
            i += 1; continue
        if s[i:i+2] == '//': in_line_comment = True; i += 2; continue
        if s[i:i+2] == '/*': in_block_comment = True; i += 2; continue
        if c == '"':
            if s[i-1:i] == 'r' or s[i-2:i] == 'br':
                pass  # raw strings: rare in this crate, not special-cased
            in_str = True; i += 1; continue
        if c == "'":
            j = i + 1
            if j < n and s[j] == '\\':
                k = j + 1
                if k < n and s[k] == 'u' and k + 1 < n and s[k+1] == '{':
                    end = s.find('}', k)
                    if end != -1 and end + 1 < n and s[end+1] == "'":
                        i = end + 2; continue
                if k + 1 < n and s[k+1] == "'":
                    i = k + 2; continue
            elif j < n and j + 1 < n and s[j+1] == "'":
                i = j + 2; continue
            i += 1; continue
        if c in "{([":
            stack.append((c, line))
        elif c in "})]":
            if not stack:
                problems.append("line %d: extra closing '%s'" % (line, c)); i += 1; continue
            top, topline = stack.pop()
            if pairs[c] != top:
                problems.append("line %d: '%s' closes '%s' opened at line %d" % (line, c, top, topline))
        i += 1
    if stack:
        problems.append("unclosed: %s" % stack[-5:])
    return problems

bad = 0
files = sys.argv[1:] or sorted(glob.glob('src/**/*.rs', recursive=True)) + sorted(glob.glob('bin/**/*.rs', recursive=True))
for f in files:
    problems = check(f)
    if problems:
        bad += 1
        print("FAIL", f)
        for p in problems:
            print("   ", p)
print("checked %d files, %d with problems" % (len(files), bad))
sys.exit(1 if bad else 0)
