# DURU

Duru (du rust) is a simple tool for identifying large files. It allows you to get recursively search for the largest files in your filesystem.

## Example

```
duru --path /home/myhome --head 10 --crop
```

Arguments:

- `path`: Sets the top directory to scan for files
- `head`: The number of files to show in the output
- `crop`: If set only the name of the file is shown, otherwise the full path is printed
