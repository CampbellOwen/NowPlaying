// Dithering.cpp : Defines the entry point for the application.
//

#include "Dithering.h"
#include "lodepng.h"

#include <cmath>
#include <cstdint>
#include <iostream>
#include <string>
#include <vector>

using namespace std;

struct Color
{
    uint8_t r;
    uint8_t g;
    uint8_t b;
    uint8_t a; // for lodepng
};

double toGammaExpanded(uint8_t c)
{
    double linear = static_cast<double>(c) / 255.0;

    if (linear <= 0.04045)
    {
        return linear / 12.92;
    }

    return pow(((linear + 0.055) / 1.055), 2.4);
}

double gammaCompress(double c)
{
    return c <= 0.0031308 ? c * 12.92 : (1.055 * pow(c, (1 / 2.4))) - 0.055; // yo wat
}

double toGrayScale(uint8_t r, uint8_t g, uint8_t b)
{
    double rG = toGammaExpanded(r);
    double gG = toGammaExpanded(g);
    double bG = toGammaExpanded(b);

    return (0.2126 * rG) + (0.7152 * gG) + (0.0722 * bG);
}

Color toRGB(double c)
{
    uint8_t cTransformed = static_cast<uint8_t>((0xFF * c) + 0.5);
    return {cTransformed, cTransformed, cTransformed, 255};
}

Color toGrayScale(const Color &c)
{
    return toRGB(toGrayScale(c.r, c.g, c.b));
}

double quantize(double c)
{
    return c < 0.5 ? 0.0 : 1.0;
}

std::vector<double> atkinson(const std::vector<double> &img, size_t w, size_t h)
{
    std::vector<double> out = img;

    for (size_t i = 0; i < (w * h); i++)
    {
        double c = out[i];
        size_t row = i / w;
        size_t col = (i % w);
        double newC = quantize(c);
        out[i] = newC;

        double quantError = c - newC;
        double errorDiffusion = 0.125 * quantError;

        if ((col + 1) < w)
        {
            out[i + 1] += errorDiffusion;
        }
        if ((col + 2) < w)
        {
            out[i + 2] += errorDiffusion;
        }
        if ((row + 1) < h)
        {
            out[i + w] += errorDiffusion;

            if ((col - 1) >= 0)
            {
                out[i + (w - 1)] += errorDiffusion;
            }

            if (col + 1 < w)
            {
                out[i + (w + 1)] += errorDiffusion;
            }
        }
        if (row + 2 < h)
        {
            out[i + (2 * w)] += errorDiffusion;
        }
    }

    return out;
}

std::vector<double> floydSteinberg(const std::vector<double> &img, size_t w, size_t h)
{
    std::vector<double> out = img;

    for (size_t i = 0; i < w * h; i++)
    {
        double c = out[i];
        size_t row = i / w;
        size_t col = (i % w);
        double newC = quantize(c);
        out[i] = newC;

        double quantError = c - newC;

        bool onRightEdge = col >= (w - 1);
        bool onBottomEdge = row >= (h - 1);

        if (!onRightEdge)
        {
            out[i + 1] += quantError * (7.0 / 16.0);
        }
        if (!onBottomEdge)
        {
            out[i + (w - 1)] += quantError * (3.0 / 16.0);
            out[i + w] += quantError * (5.0 / 16.0);
            if (!onRightEdge)
            {
                out[i + w + 1] = out[i + w + 1] + quantError * (1.0 / 16.0);
            }
        }
    }

    return out;
}

int main(int argc, char *argv[])
{
    if (argc < 2)
    {
        std::cout << "Provide a path to an image";
        return 1;
    }

    const char *filename = argv[1];
    char *outPath = "out.png";

    if (argc > 2)
    {
        outPath = argv[2];
    }

    std::vector<uint8_t> png;
    std::vector<uint8_t> image;
    uint32_t width{0};
    uint32_t height{0};
    uint32_t error = lodepng::load_file(png, filename);
    if (error)
    {
        std::cout << "Error: " << error << ": " << lodepng_error_text(error) << "\n";
        return 1;
    }

    error = lodepng::decode(image, width, height, png);
    if (error)
    {
        std::cout << "Error: " << error << ": " << lodepng_error_text(error) << "\n";
        return 1;
    }

    size_t numPixel = static_cast<size_t>(width) * static_cast<size_t>(height);

    std::vector<double> greyScale;
    for (size_t i = 0; i < numPixel; i++)
    {
        Color *c = reinterpret_cast<Color *>(&image[i * 4]);
        double grayScale = toGrayScale(c->r, c->g, c->b);
        greyScale.push_back(grayScale);
    }

    std::vector<double> compressed_grey;
    for (double c : greyScale)
    {
        compressed_grey.push_back(gammaCompress(c));
    }

    std::vector<double> dithered = atkinson(compressed_grey, width, height);
    std::vector<uint8_t> outImg;

    for (double c : dithered)
    {
        double compressed = gammaCompress(c);
        Color color = toRGB(c);
        outImg.push_back(color.r);
        outImg.push_back(color.g);
        outImg.push_back(color.b);
        outImg.push_back(color.a);
    }

    std::string out{outPath};
    error = lodepng::encode(out, outImg, width, height);
    if (error)
    {
        std::cout << "Error: " << error << ": " << lodepng_error_text(error) << "\n";
        return 1;
    }

    return 0;
}
