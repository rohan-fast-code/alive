# Alive icons

The source icon is `icon.svg` in this folder. All packaged icons are
generated from that one source image with the Tauri icon command, as the
design spec requires:

```
npm run tauri icon src-tauri/icons/icon.svg
```

That command produces the PNG and ICO files the bundle uses, including
`icon.png` in this folder. The generated PNG and ICO files are not kept
in the repository; they are build outputs. A locally generated
`icon.png` is included in the project zip for convenience.
