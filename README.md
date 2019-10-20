# Image Grid

This is a very early prototype of a image viewing grid widget. It makes use of the ggez library to render graphics. If successful, it will be refactored into a library that can be included in other projects. It is functional enough to show that it more or less works.

# Running

`image_grid --dir <directory> [--filter <regex>] [--only <regex>] [--max <count>]`

Images will be displayed at 200px wide.

The directory may contain more than just images, but all files will be parsed as if they are images. Files that fail to parse as images are not included in the display.

To exclude files that match a specific regex from being displayed, pass in one or more `filter` options.

To only display files that match specific patterns, pass in one or more `only` options.

Use the `max` option to put an upper limit on the number of images that will be displayed in the grid.

# Limitations

* Images are displayed at a fixed resolution
* No way to see full size of image
* No actions can be taken when an image is selected