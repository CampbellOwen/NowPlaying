# CMakeList.txt : CMake project for Dithering, include source and define
# project specific logic here.
#
cmake_minimum_required (VERSION 3.8)

project ("Dithering")

# Add source to this project's executable.
add_executable (Dithering "Dithering.cpp" "Dithering.h" "lodepng.cpp" "lodepng.h")

# TODO: Add tests and install targets if needed.
