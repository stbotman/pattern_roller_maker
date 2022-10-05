# Pattern Roller Maker

Simple command line tool which uses input image to generate stl file of pattern roller of that image.
This file can be 3d-printed and used in DIY and hobby applications.
Aspect ratio of images is conserved so that one need to specify either length or diameter of roller.

## Usage examples
Simple pattern roller made from cat image:
```sh
img2roller --diameter 10 cat.jpg
```
<img width="800" src="https://user-images.githubusercontent.com/4620594/194039750-67a9e938-e75a-4ca6-8693-147f2cb9ddca.png">

Pattern roller with pins on both sides:
```sh
img2roller --length 10 --pin-length 1 --pin-diameter 1 pattern.png
```
<img width="800" src="https://user-images.githubusercontent.com/4620594/194039925-b98d289b-7f65-4616-bcc3-9d71397410ee.png">

Pattern roller made by copying image 6 times along circumference with through hole added:
```sh
img2roller --stack-horizontal 6 --diameter 10 --channel-diameter 4 bark.tiff
```
<img width="800" src="https://user-images.githubusercontent.com/4620594/194040219-93f19c25-0a08-4780-ba04-544deed712e5.png">

Small input image (7px×8px) upscaled and copied 10 times in both directions:
```sh
img2roller --diameter 10 --stack-horizontal 10 --stack-vertical 10 --grid-step 0.1 --pixelated tiny.bmp
```
<img width="800" src="https://user-images.githubusercontent.com/4620594/194040362-9df4fe1c-0b68-483c-a7d3-9c4f1f4aab57.png">

## Installation
Pre-built binaries can be obtained from [Release section](https://github.com/stbotman/pattern_roller_maker/releases).
