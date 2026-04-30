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

## Optimisations
When starting the application the loading screen appears. Under the hood, it traverses all directories on a user's computer to find images.

Initially, the program creates a ***queue*** (FIFO) that contains user's drives as directories. Then, until the queue is empty, the program removes directories from the queue and reads its contents.
Other directories it finds are inserted into the queue. 

Images it finds are inserted into 4 separate data structures. The main one is a ***vector*** that contains objects of type `ImageData`, which as the name suggests contains information about an image.

The other three are ***binary tree maps*** that contain key-value pairs. In these maps, keys represent the file size, the time of creation, and the time of last modification.
There is a map for each of these key values. Values in the map are vectors with indices that act as a referrence to the main vector.

I chose to hold key-value pairs in a binary tree map, because inserted values are automatically sorted relatively quickly.

To further optimise the traversal, directories are read ***in parallel***.

## Purpose of this project

This application was created as my *CS50x* final project. I chose Rust because it seemed to be complicated and I wanted to take up a challenge. I enjoyed Rust a lot.
