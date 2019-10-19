# Image Grid

This is a very early prototype of a image viewing grid widget. It makes use of the ggez library to render graphics. If successful, it will be refactored into a library that can be included in other projects. It is functional enough to show that it more or less works.

# Running

`image_grid --dir <directory>`

The directory must only contain images. The checked in version will only display the first 10 images. The images displayed will be resized up to 100 / 100 (aspect ratio preserved).

# Limitations

* Requires that directory only contain images
* No way to override how many images are displayed
* Images are displayed at a fixed resolution
* No way to see full size of image
* No actions can be taken when an image is selected