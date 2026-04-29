# Image Explorer

This project is a desktop application that lets users explore images from their computer. It allows filtering and sorting images by various criteria.

It was created in **Rust** using **iced** library for GUI.

## Installation

To run the application on Windows, the zip file must be unpacked where the executable and resources are located. When the executable is ran, it must be in the same directory as the resources folder, otherwise the icons will not work.

## Overview

![Overview](https://2115420.xyz/static/overview.png)

The application traverses all directories on all hard drives, searches for images, and gathers paths to them. It supports images in the following formats:

- PNG
- JPEG
- GIF
- WebP
- BMP
- PNM
- TIFF
- TGA
- DDS
- ICO
- HDR
- OpenEXR
- AVIF
- QOI

The application gives information about the loading time, the number of images it found overall and the number of images in a specific format.

![Loading information](https://2115420.xyz/static/time_exec.png)

## Exploring

### Sorting

Images can be sorted by their file size, the time of their creation and the time of their modification. By clicking the arrow, sorting order can either be descending or ascending.

![Sorting](https://2115420.xyz/static/sort.png)

### Filtering

Images can be filtered by the name of a file and the encoding format.
Multiple filters can be applied at once. Filters can be combined with sorting as well.

![Filtering](https://2115420.xyz/static/filter.png)

### Viewer

Images can be viewed when an image thumbnail is clicked.
In the viewer mode an image can be zoomed and more detailed information is given, such as their path, file name, and metadata.

![Viewer mode](https://2115420.xyz/static/viewer.png)

### Navigation

Page information is displayed in the footer bar at the bottom of the screen. It includes the current page field and the number of pages.

![Navigation](https://2115420.xyz/static/navigation.png)

When using filters the number of pages will not be displayed until all available images are filtered.
The explorer will navigate to a different page when a different page number is entered in the this field.
Pages can also be navigated via keyboard arrow keys or by clicking GUI arrows in the footer bar.

While in viewer mode, images can be navigated similarly; via keyboard arrow keys or the GUI arrows.

## Purpose of this project

This application was created as my *CS50x* final project. I chose Rust because it seemed to be complicated and I wanted to take up a challenge. I enjoyed Rust a lot.
