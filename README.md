# Sizr

A basic CLI util to see file size of (web) assets after applying compression

```shell
# Only collect brotli encoding
sizr br ./lib/mod.js
```

```shell
# Pipe `fd` as file input list and get brotli, gzip, and raw sizes
sizr all `fd .js$`
```
