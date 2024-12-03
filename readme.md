yugen is a regex obfuscator.

here's how it works:

- parse regex string into an AST
- obfuscate specific patterns into more complex equivalent patterns
- convert the modified AST back to a string

todos:

- [x] build parser
- [x] obfuscate single character (`a` → `[\u{61}]`)
- [x] obfuscate character class (`[abc]` → `(?:[a]|[b]|[c])`)
- [ ] obfuscate dot (`.` → `[^\n]` (or `[\s\S]` if dot-all))
- [ ] obfuscate quantifiers
  - [ ] `*` -> `{0,}`
  - [ ] `-` → `{1,}`
  - [ ] `?` → `{0,1}`
- [ ] obfuscate groups (`(abc)` -> `?:[\u{61}][\u{62}][\u{63}])`)
- [ ] obfuscate backreferences (`\1` → `(?:\1)`)
- [ ] obfuscate lookarounds (`(?=a)` → `(?=(?:[\u{61}]))`)
