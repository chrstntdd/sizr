# Sizr

A basic CLI util to report file size of (web) assets on disk after applying compression

```shell
# Only collect brotli encoding
sizr br ./lib/mod.js

# Collect brotli & gzip encoding
sizr br,gz ./lib/mod.js

# Only collect brotli encoding & raw size
sizr br,raw ./lib/mod.js
```

```shell
# Pipe `fd` as file input list and get brotli, gzip, and raw sizes
sizr `fd '.(js|ts)$'`

# Pipe `find` and print the brotli-encoded size
sizr br `find . -name '*.js' -o -name '*.ts'`
```
